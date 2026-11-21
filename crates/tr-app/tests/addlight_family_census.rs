//! The AddLight-FAMILY census freeze (W4-g18 SEAL, S1-E-5 wave 4,
//! code-only - WRITTEN NOT RUN, no compiler invoked).
//!
//! The g18 lane's SEAL witness: the per-file `Lighting.AddLight` site
//! counts for the CORE family (the S1-E-5 wave-4 scope), frozen against
//! the decompilation so a decomp refresh or a lossy merge cannot silently
//! move/drop a site without the family registers noticing. The per-file
//! SITE-LINE freezes live in the sibling lanes' witnesses:
//! * `proj_addlight_census.rs` (W4-f7 `67110ccf`) - the 107 Projectile.cs
//!   sites verbatim (line numbers AND argument text).
//! * `main_addlight_census.rs` (W4-g19 `8005116f`) - the 5 Main.cs sites
//!   verbatim + the negative light-write census.
//! * `main_addlight_batch1.rs` (W4-g18 `a99dfa964`) - the 332-chain
//!   behavioural oracle.
//! THIS file freezes the FAMILY-LEVEL counts (the d54-bible reconcile,
//! research/366): Projectile 107 / NPC 80 / Dust 94 / Player 46 /
//! WorldItem 69 / Main 5 / TileDrawing 6, the EMPTY files (Item /
//! TileObject / Wiring / NetMessage / WorldGen), and the Lighting.cs
//! funnel gate. It reads the gitignored `decompiled/` junction at test
//! time and FAILS when the junction is absent (the S1-L-12d localization
//! precedent, the g19/f7 witnesses' `read_source`).
//!
//! The gems-NEVER-emit-light boundary (research/366 §3, the c29
//! research/260 refutation): the gem tiles 63-68/178/566 have NO
//! `Main.tileLighted` write, NO top-level `TileLightScanner.ApplyTileLight`
//! case, and the generated `TILE_LIGHTED`/`EMISSIVE` tables carry no gem
//! entry - their sparkle is dust-43 (`TileDrawing.cs:7479`), not a light.

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

/// The count of `Lighting.AddLight(` call-site tokens in a C# file.
fn addlight_count(rel: &str) -> usize {
    read_source(rel)
        .lines()
        .filter(|l| l.contains("Lighting.AddLight("))
        .count()
}

#[test]
fn the_core_addlight_family_per_file_census_is_frozen() {
    // The core family (research/366 §1/§2). Each count is the whole-file
    // `Lighting.AddLight(` token count, re-derived independently at the
    // g18 SEAL (not copied from the sibling lanes).
    assert_eq!(addlight_count("Terraria/Projectile.cs"), 107, "Projectile.cs — c19 (1-67) + e6 (68-107), the f7 freeze");
    assert_eq!(addlight_count("Terraria/NPC.cs"), 80, "NPC.cs — c20 (1-40) + e7 (41-80); the 2 indirect PlotTileLine casts are f8's, not Lighting.AddLight tokens");
    assert_eq!(addlight_count("Terraria/Dust.cs"), 94, "Dust.cs — main dust_lights (42) + g13 dust_lights_b (52) = 94/94");
    assert_eq!(addlight_count("Terraria/Player.cs"), 46, "Player.cs — main player_lights (32) + e7 gap-batch (10) + Snapping-Stone trio (3) + wing-27 cited");
    assert_eq!(addlight_count("Terraria/WorldItem.cs"), 69, "WorldItem.cs — main item_light (68) + g20 shimmer_light (:482)");
    assert_eq!(addlight_count("Terraria/Main.cs"), 5, "Main.cs — the 332/488 hook chains, e8's tile_lights.rs; g18/g19/f9");
    assert_eq!(
        addlight_count("Terraria/GameContent/Drawing/TileDrawing.cs"),
        6,
        "TileDrawing.cs — the mushroom-tree flicker (treeStyle==14), e8's mushroom_tree_light"
    );
}

#[test]
fn the_empty_files_have_no_addlight_sites() {
    // The e7 remaining-files census close (e37ec27d9): these files carry
    // ZERO Lighting.AddLight. WorldGen's one PlotTileLine (:72966) is a
    // CheckStopForSolids solid-probe, NOT a light delegate.
    for rel in [
        "Terraria/Item.cs",
        "Terraria/TileObject.cs",
        "Terraria/Wiring.cs",
        "Terraria/NetMessage.cs",
        "Terraria/WorldGen.cs",
    ] {
        assert_eq!(addlight_count(rel), 0, "{rel} must stay EMPTY of AddLight");
    }
}

#[test]
fn the_lighting_funnel_gate_is_pinned() {
    // Lighting.cs:157-163 — every overload funnels into the int form
    // behind `!Main.gamePaused && Main.netMode != 2`, appending to the
    // active engine's per-frame list. The Vector2 overloads (:135-143)
    // truncate the QUOTIENT `(int)(position.X / 16f)`; the torch-id
    // overload (:151-154) truncates the coordinate FIRST.
    let text = read_source("Terraria/Lighting.cs");
    let lines: Vec<&str> = text.lines().collect();
    let at = |n: usize| lines[n - 1].trim();
    assert_eq!(at(157), "public static void AddLight(int i, int j, float r, float g, float b)");
    assert_eq!(at(159), "if (!Main.gamePaused && Main.netMode != 2)");
    assert_eq!(at(161), "Lighting._activeEngine.AddLight(i, j, new Vector3(r, g, b));");
    assert_eq!(at(137), "Lighting.AddLight((int)(position.X / 16f), (int)(position.Y / 16f), rgb.X, rgb.Y, rgb.Z);");
    assert_eq!(at(142), "Lighting.AddLight((int)(position.X / 16f), (int)(position.Y / 16f), r, g, b);");
    assert_eq!(at(154), "Lighting.AddLight((int)position.X / 16, (int)position.Y / 16, R, G, B);");
}

#[test]
fn the_gem_tiles_have_no_tilelighted_write() {
    // The c29 gems-NEVER-emit-light refutation (research/260 Item 1,
    // re-derived in research/366 §3): Main.tileLighted[] is written for
    // ~100 ids but NEVER for the gem tiles 63-68 / 178 / 566 / 211.
    let text = read_source("Terraria/Main.cs");
    for gem in [63, 64, 65, 66, 67, 68, 178, 566, 211] {
        let needle = format!("Main.tileLighted[{gem}]");
        assert!(
            !text.contains(&needle),
            "gem tile {gem} must have NO Main.tileLighted write (the sparkle is dust-43, not a light)"
        );
    }
    // The shine CADENCE table IS present (a sparkle rate, not a light):
    assert!(text.contains("Main.tileShine[178] = 500;"), "tileShine[178]=500 (Main.cs:7516)");
    assert!(text.contains("Main.tileShine[63] = 900;"), "tileShine[63]=900 (Main.cs:7523)");
    assert!(text.contains("Main.tileShine[566] = 900;"), "tileShine[566]=900 (Main.cs:7529)");
}

#[test]
fn the_gem_sparkle_is_dust_43_not_a_light() {
    // TileDrawing.cs:7453-7482 — the 178 exposed-gem sparkle spawns dust
    // type 43 with velocity *= 0; NO Lighting.AddLight in the arm.
    let text = read_source("Terraria/GameContent/Drawing/TileDrawing.cs");
    let lines: Vec<&str> = text.lines().collect();
    // The 178 case's dust spawn (:7479) and the velocity zero (:7480).
    assert_eq!(
        lines[7478].trim(),
        "int num43 = Dust.NewDust(new Vector2(i * 16, j * 16), 16, 16, 43, 0f, 0f, 254, newColor, 0.5f);",
        "the 178 sparkle is Dust.NewDust type 43 (TileDrawing.cs:7479)"
    );
    assert_eq!(lines[7479].trim(), "this._dust[num43].velocity *= 0f;");
    // The whole DrawTiles_EmitParticles shine arm (:7413-7529) carries no
    // Lighting.AddLight — the gem sparkle is dust, not a light-map source.
    let arm: String = lines[7412..7529].join("\n");
    assert!(
        !arm.contains("Lighting.AddLight("),
        "the tileShine sparkle arm (TileDrawing.cs:7413-7529) must contain NO AddLight"
    );
}

#[test]
fn the_scanner_has_no_top_level_gem_case() {
    // TileLightScanner.ApplyTileLight's top-level `switch (tile.type)`
    // (:347) has NO case 63-68/178/566/211. The case-63-68 hits elsewhere
    // are NESTED frame-index sub-switches (chandelier/lantern styles), not
    // tile kinds. The outer gate is `if (Main.tileLighted[tile.type])`
    // (:345) — gems never enter it.
    let text = read_source("Terraria/Graphics/Light/TileLightScanner.cs");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[344].trim(), "if (Main.tileLighted[tile.type])");
    assert_eq!(lines[346].trim(), "switch (tile.type)");
    // A top-level case is indented exactly three tabs (the switch body).
    for gem in [63, 64, 65, 66, 67, 68, 178, 566, 211] {
        let top_level_case = format!("\t\t\tcase {gem}:");
        assert!(
            !lines.iter().any(|l| *l == top_level_case),
            "gem tile {gem} must have NO top-level ApplyTileLight case"
        );
    }
}
