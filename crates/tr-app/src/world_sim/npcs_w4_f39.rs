// W4-f39 census — `NPC.DoDeathEvents_AdvanceSlimeRain` (`NPC.cs:79080-
// 79096`): the per-death `slimeRain && slimeRainNPC[type] && !AnyNPCs(50)`
// gate, the 150/75 threshold + the `-num/2` hysteresis, the type-1 colour
// filter, and the King-death `StopSlimeRain` order (`NPC.cs:79405-79410`).
// Written-not-run (code-only lane).

use super::*;
use crate::app::flow::front_flow_tests::flow_app;

/// A 64x200 world with ground at row 120 — the King's `surface_of` scan
/// needs real tiles (the flow_app 8x8 world has none).
fn ground_app() -> App {
    let mut app = flow_app(None, vec![], vec![]);
    let mut big = World::new("t".into(), 64, 200);
    for x in 0..64 {
        for y in 120..200 {
            let i = big.index(x, y);
            big.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
    }
    big.compute_column_surface();
    big.world_surface = 120.0;
    big.slime_rain = true;
    big.slime_rain_time = 30000.0;
    app.world = big;
    let mut p = crate::player::Player::spawn(&app.world);
    p.x = 30.0 * TILE_PX;
    p.y = 118.0 * TILE_PX;
    app.player = Some(p);
    app
}

fn dead_slime(app: &mut App, variant: SlimeVariant) {
    let mut s = Npc::spawn_slime_sky(30.0 * TILE_PX, 90.0 * TILE_PX, variant);
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
    let mut z = Npc::new(NpcKind::Zombie, 30.0 * TILE_PX, 90.0 * TILE_PX, 1, 0.0);
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
    let mut king = Npc::new(NpcKind::KingSlime, 30.0 * TILE_PX, 118.0 * TILE_PX, 1, 0.0);
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
    let mut king = Npc::new(NpcKind::KingSlime, 30.0 * TILE_PX, 118.0 * TILE_PX, 1, 0.0);
    king.hp = 0; // dead on arrival
    app.npcs.push(king);
    dead_slime(&mut app, SlimeVariant::Blue);
    update_npcs(&mut app, 1.0 / 60.0);
    assert!(app.world.downed_slime_king, "the boss flag raised");
    assert!(!app.world.slime_rain, "the King's death stopped the rain");
    assert!(app.world.slime_rain_time < 0.0, "the negative cooldown");
    assert_eq!(app.world.slime_rain_kills, 43, "the slime counted; the stop keeps it");
}
