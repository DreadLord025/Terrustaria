// W4-f39 census — `NPC.DoDeathEvents_AdvanceSlimeRain` (`NPC.cs:79080-
// 79096`): the per-death `slimeRain && slimeRainNPC[type] && !AnyNPCs(50)`
// gate, the 150/75 threshold + the `-num/2` hysteresis, the type-1 colour
// filter, and the King-death `StopSlimeRain` order (`NPC.cs:79405-79410`).
// Written-not-run (code-only lane).

use super::*;
use crate::app::flow::front_flow_tests::flow_app;

/// A 400x400 world with ground at row 200 — wide enough for the King's
/// `SpawnOnPlayer` scan (the ±84-tile spawn area needs off-screen ground
/// columns; the old 64-wide rig clamped the area shut and the scan could
/// never find a tile).
fn ground_app() -> App {
    let mut app = flow_app(None, vec![], vec![]);
    let mut big = World::new("t".into(), 400, 400);
    for x in 0..400 {
        for y in 200..400 {
            let i = big.index(x, y);
            big.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
    }
    big.compute_column_surface();
    big.world_surface = 200.0;
    big.slime_rain = true;
    big.slime_rain_time = 30000.0;
    app.world = big;
    let mut p = crate::player::Player::spawn(&app.world);
    p.x = 200.0 * TILE_PX;
    p.y = 198.0 * TILE_PX;
    app.player = Some(p);
    app
}

fn dead_slime(app: &mut App, variant: SlimeVariant) {
    let mut s = Npc::spawn_slime_sky(200.0 * TILE_PX, 190.0 * TILE_PX, variant);
    s.hp = 0;
    app.npcs.push(s);
}

#[test]
fn w4_f39_the_counter_climbs_to_150_then_flips_negative_75() {
    // `NPC.cs:79084-79093`: `num = 150` before the first King; each
    // qualifying death `slimeRainKillCount++`; at `>= num` the King spawns
    // and the counter flips to `-num / 2` (= -75). The climb is one death
    // per tick here so the per-death gate is exercised each time.
    let mut app = ground_app();
    app.world.slime_rain_kills = 149;
    dead_slime(&mut app, SlimeVariant::Blue);
    update_npcs(&mut app, 1.0 / 60.0);
    assert_eq!(app.world.slime_rain_kills, -75, "the -num/2 hysteresis");
    assert!(
        app.npcs.iter().any(|n| n.kind == NpcKind::KingSlime),
        "the King spawned at 150"
    );
}

#[test]
fn w4_f39_the_threshold_halves_to_75_after_the_first_king() {
    // `NPC.cs:79085-79088`: `if (downedSlimeKing) num /= 2` — 75 after the
    // first King, and the flip is `-75 / 2` = -37 (integer division).
    let mut app = ground_app();
    app.world.downed_slime_king = true;
    app.world.slime_rain_kills = 74;
    dead_slime(&mut app, SlimeVariant::Blue);
    update_npcs(&mut app, 1.0 / 60.0);
    assert_eq!(app.world.slime_rain_kills, -37, "-75/2 = -37");
    assert!(app.npcs.iter().any(|n| n.kind == NpcKind::KingSlime));
}

#[test]
fn w4_f39_every_colour_variant_counts_the_type_1_filter() {
    // `Main.cs:8864`: `slimeRainNPC[1] = true` — the ONLY set slot. Every
    // colour variant shares vanilla type 1 (`NPCID.FromNetId` maps -3/-4/
    // -7/-9 all to 1), so Blue/Green/Pink/Yellow all count; a non-slime
    // (type != 1) does not.
    for variant in [
        SlimeVariant::Blue,
        SlimeVariant::Green,
        SlimeVariant::Pinky,
        SlimeVariant::Yellow,
    ] {
        let mut app = ground_app();
        app.world.slime_rain_kills = 0;
        dead_slime(&mut app, variant);
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.slime_rain_kills, 1, "{variant:?} counts");
    }
    // A zombie death (type 3) does not advance the counter.
    let mut app = ground_app();
    app.world.slime_rain_kills = 0;
    let mut z = Npc::new(NpcKind::Zombie, 200.0 * TILE_PX, 190.0 * TILE_PX, 1, 0.0);
    z.hp = 0;
    app.npcs.push(z);
    update_npcs(&mut app, 1.0 / 60.0);
    assert_eq!(app.world.slime_rain_kills, 0, "the non-slime does not count");
}

#[test]
fn w4_f39_a_live_king_closes_the_gate_for_every_death() {
    // `NPC.cs:79082`'s `!NPC.AnyNPCs(50)`: while a King lives, NO slime
    // death counts — the counter stays put across a whole batch.
    let mut app = ground_app();
    app.world.slime_rain_kills = 10;
    let mut king = Npc::new(NpcKind::KingSlime, 200.0 * TILE_PX, 198.0 * TILE_PX, 1, 0.0);
    king.hp = king.max_hp;
    app.npcs.push(king);
    for _ in 0..5 {
        dead_slime(&mut app, SlimeVariant::Blue);
    }
    update_npcs(&mut app, 1.0 / 60.0);
    assert_eq!(app.world.slime_rain_kills, 10, "the live King gated them all");
}

#[test]
fn w4_f39_a_king_spawned_mid_batch_closes_the_gate_for_later_deaths() {
    // W4-f39's per-death fix: the C# re-evaluates `!AnyNPCs(50)` for EACH
    // death (`:79082`). Two slimes die in one tick with the counter at
    // 149: the FIRST hits 150, spawns the King and flips to -75; the
    // SECOND sees the now-live King and does NOT count (the pre-fix port
    // checked the gate once before the loop and would have counted it,
    // landing -74).
    let mut app = ground_app();
    app.world.slime_rain_kills = 149;
    dead_slime(&mut app, SlimeVariant::Blue);
    dead_slime(&mut app, SlimeVariant::Green);
    update_npcs(&mut app, 1.0 / 60.0);
    assert_eq!(
        app.world.slime_rain_kills, -75,
        "the second death saw the mid-batch King and did not count"
    );
    assert_eq!(
        app.npcs.iter().filter(|n| n.kind == NpcKind::KingSlime).count(),
        1,
        "exactly one King"
    );
}

#[test]
fn w4_f39_the_king_death_stops_the_rain_and_the_counter_survives() {
    // `NPC.cs:79405-79410` (case 50): the King's death while the rain is
    // up calls `StopSlimeRain` (the negative cooldown) and fires
    // progression 16. The kill counter is NOT cleared by the stop
    // (`Main.cs:64787-64820`). The port runs the counter pass ahead of the
    // boss-death `checkDead` pass, so a slime dying in the SAME tick as
    // the King still counts (the C# interleaves per-NPC; the loop split is
    // the recorded deviation) — the counter here advances 42 → 43.
    let mut app = ground_app();
    app.world.slime_rain_kills = 42;
    let mut king = Npc::new(NpcKind::KingSlime, 200.0 * TILE_PX, 198.0 * TILE_PX, 1, 0.0);
    king.hp = 0; // dead on arrival
    app.npcs.push(king);
    dead_slime(&mut app, SlimeVariant::Blue);
    update_npcs(&mut app, 1.0 / 60.0);
    assert!(app.world.downed_slime_king, "the boss flag raised");
    assert!(!app.world.slime_rain, "the King's death stopped the rain");
    assert!(app.world.slime_rain_time < 0.0, "the negative cooldown");
    assert_eq!(app.world.slime_rain_kills, 43, "the slime counted; the stop keeps it");
}

#[test]
fn w4_f39_end_to_end_rain_then_150_kills_then_the_king_appears() {
    // The closeout witness the Lead asked for: the rain is UP, 150 slime
    // deaths climb the counter one per tick (the per-death gate open
    // every time — no King alive), and at the 150th the King APPEARS in
    // the roster via the real `SpawnOnPlayer` scan (not just the counter
    // flip). The King lands on the ground row (bottom-centre anchor at
    // y*16, `NPC.cs:80770`), targets the player and carries the
    // `timeLeft *= 20` window (`:80686`).
    let mut app = ground_app();
    // The HasAwoken line reads the shipped English resources (the
    // producer tests' init convention + the culture lock — the emotes
    // tests switch cultures process-wide).
    let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
    crate::lang::init(tr_core::localization::GameCulture::English);
    assert!(app.world.slime_rain);
    assert_eq!(app.world.slime_rain_kills, 0);
    for kill in 1..=150 {
        dead_slime(&mut app, SlimeVariant::Blue);
        update_npcs(&mut app, 1.0 / 60.0);
        if kill < 150 {
            assert_eq!(app.world.slime_rain_kills, kill, "kill {kill} counted");
            assert!(
                !app.npcs.iter().any(|n| n.kind == NpcKind::KingSlime && n.hp > 0),
                "no King before the threshold"
            );
        }
    }
    assert_eq!(app.world.slime_rain_kills, -75, "the flip at 150");
    let king = app
        .npcs
        .iter()
        .find(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
        .expect("the King appeared at the 150th kill");
    // The scan's ground-row anchor: the King's bottom sits on a ground
    // row (200..400) — `y*16` with y the first solid row below the
    // candidate. The spawn area's rows are 146..250, so the ground hit
    // is row 200 (the flat plain) — bottom = 3200.
    let (_, h) = king.size();
    assert_eq!(king.y + h, 200.0 * TILE_PX, "the King stands on the ground row");
    assert_eq!(king.target, 0, "SpawnBoss targets the player");
    assert_eq!(king.despawn_timer, 937 * 20, "timeLeft *= 20");
    assert!(king.boss, "SetDefaults case 50's boss flag");
    // The announcement broadcast (`NPC.cs:80703`).
    assert!(
        app.chat
            .messages
            .iter()
            .any(|l| l.text.contains("has awoken")),
        "the HasAwoken broadcast"
    );
}

#[test]
fn w4_f39_the_spawn_scan_lands_off_screen_and_the_stream_advances() {
    // `SpawnOnPlayer`'s scan (`NPC.cs:80577-80628`): the candidate tiles
    // draw `Next(spawnArea.Left, Right)` + `Next(Top, Bottom)` pairs on
    // the caller's stream; the accepted tile is OFF the player's
    // inflated screen rect (`CheckNotSpawningOnScreen`, `:5436-5450`).
    // The player sits at tile 200; the rect spans ±(960+62) px (the
    // safe ranges ride RAW as pixels, `:5444`) → tiles ~135..265. The
    // King must land outside it.
    let mut app = ground_app();
    let p = app.player.as_ref().unwrap();
    let pos = (p.x, p.y);
    let center = p.center();
    let mut rng = UnifiedRandom::new(12345);
    let pristine = rng.clone();
    let king = spawner::spawn_king_slime_on_player(&app.world, pos, center, &mut rng)
        .expect("the flat plain gives the scan a tile");
    assert_ne!(rng, pristine, "the scan drew candidate tiles");
    let (cx, _) = center;
    let king_cx = king.x + king.size().0 * 0.5;
    // Off-screen: outside the inflated screen rect (half-width 960 +
    // the RAW 62-px safe range; the tile-rect test is on the 16-px
    // cell, the centre is within 8 px of it).
    assert!(
        (king_cx - cx).abs() > (spawner::S_WIDTH / 2 + 62) as f32,
        "the King landed off the player's screen ({king_cx} vs {cx})"
    );
    // The headroom pair passed: 20 tiles of open air above the ground
    // row and a clear line to the player (the flat plain has both).
    assert_eq!(king.y + king.size().1, 200.0 * TILE_PX);
}

#[test]
fn w4_f39_a_walled_world_refuses_the_scan_and_the_counter_still_flips() {
    // The C# `SpawnOnPlayer` can find NO tile (every candidate refused);
    // `:79093` still flips the counter — the spawn is best-effort, the
    // hysteresis is not. A world whose every column is solid from row 0
    // leaves no open candidate cell: the scan spends its draws and
    // returns None.
    let mut app = ground_app();
    for x in 0..400 {
        for y in 0..200 {
            let i = app.world.index(x, y);
            app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
    }
    app.world.compute_column_surface();
    app.world.slime_rain_kills = 149;
    dead_slime(&mut app, SlimeVariant::Blue);
    update_npcs(&mut app, 1.0 / 60.0);
    assert_eq!(app.world.slime_rain_kills, -75, "the flip runs regardless");
    assert!(
        !app.npcs.iter().any(|n| n.kind == NpcKind::KingSlime && n.hp > 0),
        "the fully-solid world gave the scan no tile"
    );
}
