// W4-f39 census — `Spawner.SlimeRainSpawns` (`NPC.cs:5892-5955`): the
// position.Y depth gate, the nearbyActiveNPCs 0.65 slime discount, the
// `45 + (int)(450f*(nearby/15))` denominator, the sky window, the
// SolidTiles + house-wall validity scan, and the four-arm kind chain
// (the Next(200) Pinky included). Written-not-run (code-only lane).
// Every RNG pin mirrors the draw sequence on a cloned UnifiedRandom.

use super::*;
use crate::app::flow::front_flow_tests::flow_app;

/// A 400x400 world, ground at row 200, surface 200 — the sky window
/// (`[surface*0.3, surface]` = rows 60..200) is open air above the ground.
fn sky_app() -> App {
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
    p.y = 198.0 * TILE_PX; // just above the ground, well under the depth gate
    app.player = Some(p);
    app.npc_timer = 0.0; // let the ambient spawn arm run
    app
}

/// Mirror the full SlimeRainSpawns draw sequence on a twin stream: the
/// `Next(num2)` gate, the two window draws, then the kind chain. Returns
/// the expected variant for a hit (or None when the gate misses).
fn replay_spawn(seed: i32, nearby: f32, pcx: f32, pcy: f32) -> (bool, Option<SlimeVariant>) {
    let mut r = UnifiedRandom::new(seed);
    let num2 = 45 + (450.0f32 * (nearby / 15.0)) as i32;
    if r.next_max(num2) != 0 {
        return (false, None);
    }
    let num3 = (pcx - 1920.0) as i32;
    let _x = r.next_range(num3, num3 + 3840);
    let _y = r.next_range(
        (pcy as f64 - 1800.0) as i32,
        (pcy as f64 - 900.0) as i32,
    );
    // The kind chain (the validity scan is RNG-free).
    let variant = if r.next_max(200) == 0 {
        SlimeVariant::Pinky
    } else if r.next_max(10) == 0 {
        SlimeVariant::Purple
    } else if r.next_max(5) < 2 {
        SlimeVariant::Green
    } else {
        SlimeVariant::Blue
    };
    (true, Some(variant))
}

#[test]
fn w4_f39_the_depth_gate_reads_position_y_not_the_centre() {
    // `NPC.cs:5897`: `player.position.Y > worldSurface*16 + 1200/2` — the
    // TOP of the player, not the centre. surface 200 → the gate is
    // `y > 3200 + 600 = 3800`. A player at y 3799 passes the gate (draws
    // the Next(num2)); y 3801 refuses before any draw. The gate is the
    // RNG-free observable (the spawn itself may still miss/refuse on the
    // window tile — the deep window sits in the ground here).
    for (y, passes) in [(3799.0f32, true), (3801.0, false)] {
        let mut app = sky_app();
        app.player.as_mut().unwrap().y = y;
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        let mut rng = app.rngs.stream(RngKey::SpawnNpc);
        super::slime_rain_spawns(&mut app, &mut rng);
        if passes {
            assert_ne!(
                *app.rngs.stream(RngKey::SpawnNpc),
                pristine,
                "y {y}: under the gate, the Next(num2) drew"
            );
        } else {
            assert_eq!(
                *app.rngs.stream(RngKey::SpawnNpc),
                pristine,
                "y {y}: the depth gate refused before any draw"
            );
        }
    }
}

#[test]
fn w4_f39_the_crowd_gate_is_nearby_gt_15_with_the_slime_discount() {
    // `NPC.cs:5897`'s `nearbyActiveNPCs > 15f` + `:77916-77918`'s 0.65
    // slime discount: 24 type-1 slimes count 24*0.65 = 15.6 > 15 (refuse);
    // 23 count 14.95 <= 15 (pass). Without the discount 16 slimes would
    // already refuse — the discount widens the crowd window. The gate is
    // RNG-free observable: a refusal draws NOTHING; a pass draws at least
    // the Next(num2) gate (the spawn itself may still miss/refuse).
    for (count, passes) in [(23usize, true), (24, false)] {
        let mut app = sky_app();
        let (pcx, pcy) = app.player.as_ref().unwrap().center();
        for _ in 0..count {
            // Screen-rect slimes around the player (within 960x600).
            app.npcs
                .push(Npc::spawn_slime_sky(pcx, pcy, SlimeVariant::Blue));
        }
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        let mut rng = app.rngs.stream(RngKey::SpawnNpc);
        super::slime_rain_spawns(&mut app, &mut rng);
        if passes {
            assert_ne!(
                *app.rngs.stream(RngKey::SpawnNpc),
                pristine,
                "{count} slimes (14.95) passed the gate and drew"
            );
        } else {
            assert_eq!(
                *app.rngs.stream(RngKey::SpawnNpc),
                pristine,
                "{count} slimes (15.6) refused before any draw"
            );
        }
    }
}

#[test]
fn w4_f39_the_denominator_truncates_the_f32_product() {
    // `NPC.cs:5901`: `45 + (int)(450f * (nearby/15))`. One nearby slime →
    // nearby 0.65 → 450*(0.65/15) = 19.5 → (int)19 → num2 = 64 (NOT 65 —
    // the f32 product truncates, it does not round). The gate draw is
    // Next(num2); the stream-equality pin proves the denominator read the
    // crowd (a constant 45 would spend a different draw).
    let mut app = sky_app();
    let (pcx, pcy) = app.player.as_ref().unwrap().center();
    app.npcs
        .push(Npc::spawn_slime_sky(pcx, pcy, SlimeVariant::Blue));
    let nearby = 0.65f32;
    let num2 = 45 + (450.0f32 * (nearby / 15.0)) as i32;
    assert_eq!(num2, 64, "the f32 product truncates: 450*0.04333=19.5→19");
    // The twin spends Next(64) + the window + the kind chain; the live
    // arm spends the same sequence (the validity scan is RNG-free).
    let mut twin = UnifiedRandom::new(app.world.seed as i32);
    replay_spawn_twin(&mut twin, nearby, pcx, pcy);
    let mut rng = app.rngs.stream(RngKey::SpawnNpc);
    super::slime_rain_spawns(&mut app, &mut rng);
    assert_eq!(
        *app.rngs.stream(RngKey::SpawnNpc),
        twin,
        "the crowd-scaled Next(64) sequence matches the twin"
    );
}

#[test]
fn w4_f39_the_kind_chain_includes_the_next200_pinky() {
    // `NPC.cs:5931-5953`: `Next(200)==0` → Pinky(-4) BEFORE the classic
    // chain. A seed whose first post-window draw hits Next(200)==0 spawns
    // a Pinky; the pre-fix port folded this arm and would have drawn the
    // classic chain instead (a different variant + a shifted stream).
    // Search for a seed that ALSO lands the spawn in valid air (the
    // window tile in band, non-solid, no house wall) so the variant is
    // observable on the roster.
    let mut app = sky_app();
    let (pcx, pcy) = app.player.as_ref().unwrap().center();
    let num3 = (pcx - 1920.0) as i32;
    let ylo = (pcy as f64 - 1800.0) as i32;
    let yhi = (pcy as f64 - 900.0) as i32;
    let fire = (1..6_000_000i32)
        .find(|&s| {
            let mut r = UnifiedRandom::new(s);
            if r.next_max(45) != 0 {
                return false;
            }
            let xt = r.next_range(num3, num3 + 3840) / 16;
            let yt = r.next_range(ylo, yhi) / 16;
            // The validity scan (RNG-free): in band, open air, no wall.
            if xt < 10
                || xt > 410
                || (yt as f64) < 60.0
                || (yt as f64) > 200.0
                || app.world.tile(xt, yt).is_some_and(|t| t.active())
            {
                return false;
            }
            r.next_max(200) == 0
        })
        .expect("a pinky seed with a valid window exists");
    app.rngs.reset(fire);
    let mut rng = app.rngs.stream(RngKey::SpawnNpc);
    super::slime_rain_spawns(&mut app, &mut rng);
    let spawned = app.npcs.last().expect("the spawn fired");
    assert_eq!(spawned.slime, SlimeVariant::Pinky, "the Next(200) arm");
    assert_eq!(spawned.max_hp, 150, "Pinky's 150 life (`NPC.cs:7693`)");
}

#[test]
fn w4_f39_the_full_draw_sequence_matches_the_twin_replay() {
    // The whole arm's draw order: Next(num2), Next(x-window), Next(y-
    // window), Next(200), [Next(10)], [Next(5)]. A fire seed's spawn
    // variant + the spent stream match the hand-mirrored replay exactly.
    let fire = (1..2_000_000i32)
        .find(|&s| UnifiedRandom::new(s).next_max(45) == 0)
        .expect("a fire seed exists");
    let mut app = sky_app();
    let (pcx, pcy) = app.player.as_ref().unwrap().center();
    app.rngs.reset(fire);
    let mut rng = app.rngs.stream(RngKey::SpawnNpc);
    super::slime_rain_spawns(&mut app, &mut rng);
    let (hit, want_variant) = replay_spawn(fire, 0.0, pcx, pcy);
    assert!(hit, "the gate fired");
    // The validity scan may refuse the spawn (the window tile is solid or
    // out of band) — when it lands, the variant matches the replay.
    if let Some(spawned) = app.npcs.last() {
        if spawned.kind == NpcKind::Slime {
            assert_eq!(Some(spawned.slime), want_variant, "the kind chain matches");
        }
    }
    // The stream spent exactly the replay's draws (the validity scan is
    // RNG-free, so a refusal still spends the kind-chain draws).
    let mut twin = UnifiedRandom::new(fire);
    let _ = replay_spawn_twin(&mut twin, 0.0, pcx, pcy);
    assert_eq!(
        *app.rngs.stream(RngKey::SpawnNpc),
        twin,
        "the draw sequence matches the twin"
    );
}

/// The replay's draw consumption only (for the stream-equality pin).
fn replay_spawn_twin(r: &mut UnifiedRandom, nearby: f32, pcx: f32, pcy: f32) {
    let num2 = 45 + (450.0f32 * (nearby / 15.0)) as i32;
    if r.next_max(num2) != 0 {
        return;
    }
    let num3 = (pcx - 1920.0) as i32;
    let _x = r.next_range(num3, num3 + 3840);
    let _y = r.next_range(
        (pcy as f64 - 1800.0) as i32,
        (pcy as f64 - 900.0) as i32,
    );
    if r.next_max(200) == 0 {
        return;
    }
    if r.next_max(10) == 0 {
        return;
    }
    let _ = r.next_max(5);
}

#[test]
fn w4_f39_the_house_wall_refuses_the_spawn() {
    // `NPC.cs:5926`'s `Main.wallHouse[tile[xt, yt].wall]`: a house wall
    // behind the sky tile refuses the spawn even when the air is open.
    // Fill the whole sky window with a house wall (wood wall 4) — every
    // window tile then refuses, so a fire seed draws the full sequence
    // but spawns nothing.
    let fire = (1..2_000_000i32)
        .find(|&s| UnifiedRandom::new(s).next_max(45) == 0)
        .expect("a fire seed exists");
    let mut app = sky_app();
    for x in 0..400 {
        for y in 60..200 {
            let i = app.world.index(x, y);
            app.world.tiles[i].wall = 4; // wood wall — a house wall
        }
    }
    app.rngs.reset(fire);
    let before = app.npcs.len();
    let mut rng = app.rngs.stream(RngKey::SpawnNpc);
    super::slime_rain_spawns(&mut app, &mut rng);
    assert_eq!(app.npcs.len(), before, "the house wall refused every tile");
}
