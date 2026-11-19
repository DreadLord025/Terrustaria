//! The Pumpkin Moon boss AI machines (S1-F-13b2 slice B) — `NPC.AI`'s
//! aiStyle-57 block (`NPC.cs:32761-33043`, the Mourning Wood / Everscream
//! floater) and aiStyle-58 block (`NPC.cs:33044-33197`, the Pumpking).
//!
//! The kinds landed with S1-F-13b on documented stand-in routings (the
//! hover family); this module ports the named machines verbatim and the
//! style axis in `npc.rs` claims styles 57/58 for these two kinds only
//! (the Retinazer's own aiStyle-30 routing and the aiStyle-26/22 kinds —
//! another worker's slice — are untouched).
//!
//! Faithful details kept: the hp-band speed (`num836` 2/3/4, day 8), the
//! `(vx*20 ± num836)/21` approach, the `Main.rand` draw ORDER inside every
//! shot arm (aim jitters → speed jitters → kind pick — C# evaluates the
//! named arguments in textual order, so `Type: Main.rand.Next(326, 329)`
//! draws AFTER the SpeedX/SpeedY jitters), the player-TOP aim
//! (`Main.player[target].position.Y`, not the centre), the 80×20
//! `Collision.SolidCollision` probe under the Wood, the Pumpking's
//! `localAI` frame cycle, the 300-tick `ai[3]` attack pick
//! (`Main.rand.Next(3)`), the Greek-fire arm's `WorldGen.SolidTile` origin
//! gate and `(dy + Next(50,201)) * 0.2` lob, and the `num875`/15 hover
//! lerp with its 6/8/10/12 distance bands.
//!
//! Documented deviations (ANALYSIS, S1-F-13b2 slice B):
//! * the `Lighting.AddLight` lines (`:32781`/`:32785`/`:33059`) and the
//!   type-344 dust arm (`:32821-32825`) are skipped — the same call as the
//!   Retinazer's port (`npc.rs`'s `update_retinazer` docs): no dust/light
//!   layer on these machines yet; neither draws `Main.rand` outside the
//!   skipped dust arm, so the stream is untouched;
//! * the type-344 (Everscream) branches are not ported — the kind does not
//!   exist in the roster (S1-F-13c); only the type-325 arms run;
//! * the Pumpking's two `PumpkingBlade` (328, aiStyle 59) spawns
//!   (`:33093-33103`) are deferred — the blade kind is unported; the
//!   `NPC.NewNPC` calls draw no `Main.rand` outside the `getGoodWorld`
//!   variant roll (`NPC.cs:80752`), which the port's spawn plumbing does
//!   not model (`variants.rs`), so skipping them keeps the stream exact.

use crate::npc::Npc;
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// `NPC.TargetClosest` (`NPC.cs:77652-77781`) through the shared selection
/// (`npc_target.rs`, S1-D-3p) — one session player, the zombie wiring's
/// candidate shape (`npc.rs`'s `update_zombie`).
fn target_closest(npc: &mut Npc, world: &World, player: &Player, ai_style: i32) {
    let cand = crate::npc_target::TargetCandidate {
        active: true,
        dead: player.dead,
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    crate::npc_target::target_closest(npc, world, &[cand], true, ai_style, 0);
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70133`): the full solid
/// block — active, solid, not top-only, no half-brick, no slope, not
/// framed off. The C#'s null guard (`:70115-70118`) reads a null slot as
/// solid, but the port's worlds fill every in-bounds slot (the vanilla
/// array does too); an OUT-OF-BOUNDS read throws into the catch (`:70128`)
/// → false, exactly like `grapple.rs::solid_tile`.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && tr_core::tiledata::TILE_SOLID.has(t.kind)
            && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `NPC.AI`'s aiStyle-57 block, the type-325 (Mourning Wood) arms
/// (`NPC.cs:32761-33043`). The floater: the hp-band horizontal speed, the
/// five-state attack cycle (idle → stomp/Greek-fire spin → beam/barrage),
/// the horizontal damp/approach and the vertical hover over the 80×20
/// solid probe. Ends with the `noTileCollide` integration
/// (`NPC.cs:90495-90498`).
pub fn mourning_wood_step(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    // `:32763-32765` — `num836 = 2f`, the gravity/tile flags off (the
    // port's integration below is the raw one).
    let mut num836 = 2.0f32;
    // `:32766-32769` — retarget only at night.
    if !world.day_time {
        target_closest(npc, world, player, 57);
    }
    let (w, h) = npc.size();
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let (pcx, _) = player.center();
    let mut flag61 = false;
    // `:32771-32778` — the hp bands (the C#'s double compares).
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.75 {
        num836 = 3.0;
    }
    if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
        num836 = 4.0;
    }
    // `:32783-32786` — the type-325 light is skipped (module docs).
    if world.day_time {
        // `:32787-32791` — dawn: the despawn window and the flee speed.
        npc.encourage_despawn(10);
        num836 = 8.0;
    } else if npc.ai[0] == 0.0 {
        // `:32792-32816` — the idle: the counter runs faster the lower
        // the hp; at 300 the next attack is picked. `Main.netMode != 1`
        // is the port's single-player truth.
        npc.ai[1] += 1.0;
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.5 {
            npc.ai[1] += 1.0;
        }
        if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
            npc.ai[1] += 1.0;
        }
        if npc.ai[1] >= 300.0 {
            npc.ai[1] = 0.0;
            // `:32806-32813` — type 325 is not 344, so the low-hp arm
            // picks from {3, 4}, otherwise {1, 2}.
            if (npc.hp as f64) < (npc.max_hp as f64) * 0.25 {
                npc.ai[0] = rng.next_range(3, 5) as f32;
            } else {
                npc.ai[0] = rng.next_range(1, 3) as f32;
            }
            npc.net_update = true;
        }
    } else if npc.ai[0] == 1.0 {
        // `:32846-32865` — the type-325 stomp: the flaming wood (325)
        // every 15 ticks, aimed at the player's TOP-centre at speed 10.
        flag61 = true;
        npc.ai[1] += 1.0;
        if npc.ai[1] % 15.0 == 0.0 {
            let ox = cx;
            let oy = cy + 30.0;
            let mut num842 = player.x + crate::player::WIDTH * 0.5 - ox;
            let mut num843 = player.y - oy;
            let num844 = 10.0 / (num842 * num842 + num843 * num843).sqrt();
            num842 *= num844;
            num843 *= num844;
            let sx = num842 * (1.0 + rng.next_range(-20, 21) as f32 * 0.01);
            let sy = num843 * (1.0 + rng.next_range(-20, 21) as f32 * 0.01);
            npc.wants_hostile_shot
                .push((crate::projectile::FLAMING_WOOD, ox, oy, sx, sy, 50));
        }
        if npc.ai[1] >= 120.0 {
            npc.ai[1] = 0.0;
            npc.ai[0] = 0.0;
        }
    } else if npc.ai[0] == 2.0 {
        // `:32893-32920` — the type-325 Greek-fire spin: every 8th tick
        // strictly between 60 and 240, the lobbed fire (326-328).
        flag61 = true;
        npc.ai[1] += 1.0;
        if npc.ai[1] > 60.0 && npc.ai[1] < 240.0 && npc.ai[1] % 8.0 == 0.0 {
            let mut num851 = 10.0f32;
            let ox = cx;
            let oy = cy + 30.0;
            let mut num852 = player.x + crate::player::WIDTH * 0.5 - ox;
            let mut num853 = player.y - oy - num852.abs() * 0.3;
            num851 += num852.abs() * 0.004;
            if num851 > 14.0 {
                num851 = 14.0;
            }
            num852 += rng.next_range(-50, 51) as f32;
            num853 -= rng.next_range(50, 201) as f32;
            let num854 = num851 / (num852 * num852 + num853 * num853).sqrt();
            num852 *= num854;
            num853 *= num854;
            // `:32913` — the draw order: the aim jitters above, THEN the
            // SpeedX/SpeedY jitters, THEN the kind pick (the C#'s named
            // arguments evaluate in textual order — `Type:` sits after
            // `SpeedX:`/`SpeedY:` in the call).
            let sx = num852 * (1.0 + rng.next_range(-30, 31) as f32 * 0.01);
            let sy = num853 * (1.0 + rng.next_range(-30, 31) as f32 * 0.01);
            let kind = rng.next_range(326, 329) as u16;
            npc.wants_hostile_shot.push((kind, ox, oy, sx, sy, 40));
        }
        if npc.ai[1] >= 300.0 {
            npc.ai[1] = 0.0;
            npc.ai[0] = 0.0;
        }
    } else if npc.ai[0] == 3.0 {
        // `:32922-32941` — the beam: speed 4, the flaming wood every 30
        // ticks at speed 16 with the tight 0.001 jitter, damage 75.
        num836 = 4.0;
        npc.ai[1] += 1.0;
        if npc.ai[1] % 30.0 == 0.0 {
            let ox = cx;
            let oy = cy + 30.0;
            let mut num856 = player.x + crate::player::WIDTH * 0.5 - ox;
            let mut num857 = player.y - oy;
            let num858 = 16.0 / (num856 * num856 + num857 * num857).sqrt();
            num856 *= num858;
            num857 *= num858;
            let sx = num856 * (1.0 + rng.next_range(-20, 21) as f32 * 0.001);
            let sy = num857 * (1.0 + rng.next_range(-20, 21) as f32 * 0.001);
            npc.wants_hostile_shot
                .push((crate::projectile::FLAMING_WOOD, ox, oy, sx, sy, 75));
        }
        if npc.ai[1] >= 120.0 {
            npc.ai[1] = 0.0;
            npc.ai[0] = 0.0;
        }
    } else if npc.ai[0] == 4.0 {
        // `:32942-32969` — the barrage: speed 4, the Greek fires every
        // 10th tick, the 12+|dx|*0.002 (cap 16) lob, damage 50.
        num836 = 4.0;
        npc.ai[1] += 1.0;
        if npc.ai[1] % 10.0 == 0.0 {
            let mut num860 = 12.0f32;
            let ox = cx;
            let oy = cy + 30.0;
            let mut num861 = player.x + crate::player::WIDTH * 0.5 - ox;
            let mut num862 = player.y - oy - num861.abs() * 0.2;
            num860 += num861.abs() * 0.002;
            if num860 > 16.0 {
                num860 = 16.0;
            }
            num861 += rng.next_range(-50, 51) as f32;
            num862 -= rng.next_range(50, 201) as f32;
            let num863 = num860 / (num861 * num861 + num862 * num862).sqrt();
            num861 *= num863;
            num862 *= num863;
            // `:32962` — the same textual-order draws as the spin: the
            // SpeedX/SpeedY jitters, THEN the kind pick.
            let sx = num861 * (1.0 + rng.next_range(-30, 31) as f32 * 0.005);
            let sy = num862 * (1.0 + rng.next_range(-30, 31) as f32 * 0.005);
            let kind = rng.next_range(326, 329) as u16;
            npc.wants_hostile_shot.push((kind, ox, oy, sx, sy, 50));
        }
        if npc.ai[1] >= 240.0 {
            npc.ai[1] = 0.0;
            npc.ai[0] = 0.0;
        }
    }
    // `:32970-32992` — the horizontal damp above the player and the
    // `(vx*20 ± num836)/21` approach.
    if (cx - pcx).abs() < 50.0 {
        flag61 = true;
    }
    if flag61 {
        npc.vx *= 0.9;
        if npc.vx > -0.1 && npc.vx < 0.1 {
            npc.vx = 0.0;
        }
    } else {
        if npc.facing > 0 {
            npc.vx = (npc.vx * 20.0 + num836) / 21.0;
        }
        if npc.facing < 0 {
            npc.vx = (npc.vx * 20.0 - num836) / 21.0;
        }
    }
    // `:32993-33042` — the vertical hover: the player-overlap drop, the
    // 80×20 `Collision.SolidCollision` probe (`npc.rs`'s
    // `Player::solid_collision` is the port) and the fall/rise ramps.
    let probe_x = cx - (80 / 2) as f32;
    let probe_y = npc.y + h - 20.0;
    let over_player = npc.x < player.x
        && npc.x + w > player.x + crate::player::WIDTH
        && npc.y + h < player.y + player.height - 16.0;
    if over_player {
        npc.vy += 0.5;
    } else if Player::solid_collision(world, probe_x, probe_y, 80.0, 20.0) {
        if npc.vy > 0.0 {
            npc.vy = 0.0;
        }
        if npc.vy > -0.2 {
            npc.vy -= 0.025;
        } else {
            npc.vy -= 0.2;
        }
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    } else {
        if npc.vy < 0.0 {
            npc.vy = 0.0;
        }
        if npc.vy < 0.1 {
            npc.vy += 0.025;
        } else {
            npc.vy += 0.5;
        }
    }
    if npc.vy > 10.0 {
        npc.vy = 10.0;
    }
    // `NPC.cs:90495-90498` — the noTileCollide integration.
    npc.x += npc.vx;
    npc.y += npc.vy;
}

/// `NPC.AI`'s aiStyle-58 block (`NPC.cs:33044-33197`) — the Pumpking: the
/// `localAI` frame cycle, the 300-tick attack-mode pick, the Greek-fire
/// lob while mode 0, the hover (`num875`/15) / scythe-charge (16/50) /
/// flee states and the velocity tilt. The blade spawns are deferred
/// (module docs). Ends with the `noTileCollide` integration.
pub fn pumpking_step(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    // `:33046-33055` — the frame cycle: localAI[0] counts 7 ticks,
    // localAI[1] cycles 0..4.
    npc.local_ai[0] += 1.0;
    if npc.local_ai[0] > 6.0 {
        npc.local_ai[0] = 0.0;
        npc.local_ai[1] += 1.0;
        if npc.local_ai[1] > 4.0 {
            npc.local_ai[1] = 0.0;
        }
    }
    let (w, h) = npc.size();
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let (mut pcx, mut pcy) = player.center();
    let mut ptop = player.y;
    // `:33057-33060` — the type-327 light is skipped (module docs).
    // `:33061-33086` — the attack-mode timer (`Main.netMode != 1` is the
    // port's single-player truth).
    npc.local_ai[2] += 1.0;
    if npc.local_ai[2] > 300.0 {
        npc.local_ai[2] = 0.0;
        npc.ai[3] = rng.next_max(3) as f32;
        npc.net_update = true;
    } else if npc.ai[3] == 0.0 && npc.local_ai[2] % 30.0 == 0.0 && npc.local_ai[2] > 30.0 {
        // `:33070-33085` — the Greek-fire lob: only from a non-solid
        // origin tile; the `(dy + Next(50,201)) * 0.2` flattening.
        let num867 = 5.0f32;
        let ox = cx;
        let oy = cy + 30.0;
        if !solid_tile(world, (ox as i32) / 16, (oy as i32) / 16) {
            let mut num868 = pcx - ox;
            let mut num869 = ptop - oy;
            num868 += rng.next_range(-50, 51) as f32;
            num869 = (num869 + rng.next_range(50, 201) as f32) * 0.2;
            let num870 = num867 / (num868 * num868 + num869 * num869).sqrt();
            num868 *= num870;
            num869 *= num870;
            // `:33083` — the textual-order draws: the SpeedX/SpeedY
            // jitters, THEN the kind pick (`Type:` sits last in the call).
            let sx = num868 * (1.0 + rng.next_range(-30, 31) as f32 * 0.01);
            let sy = num869 * (1.0 + rng.next_range(-30, 31) as f32 * 0.01);
            let kind = rng.next_range(326, 329) as u16;
            npc.wants_hostile_shot.push((kind, ox, oy, sx, sy, 40));
        }
    }
    // `:33087-33104` — the one-time init: retarget, arm the hover state
    // and spawn the two blades (deferred — module docs; no rand draws).
    if npc.ai[0] == 0.0 {
        target_closest(npc, world, player, 58);
        npc.ai[0] = 1.0;
        npc.net_update = true;
    }
    // `:33105-33117` — the dead/far retarget: beyond 2000 px on either
    // axis (or a dead target) the boss gives up and flees.
    if player.dead || (npc.x - player.x).abs() > 2000.0 || (npc.y - player.y).abs() > 2000.0 {
        target_closest(npc, world, player, 58);
        (pcx, pcy) = player.center();
        // The C# re-reads `player13` here (`:33108`); `Top.Y` rides the
        // lob arm, which already ran this tick — the re-read is kept for
        // the 1:1 shape.
        #[allow(unused_assignments)]
        {
            ptop = player.y;
        }
        if player.dead || (npc.x - player.x).abs() > 2000.0 || (npc.y - player.y).abs() > 2000.0 {
            if npc.ai[1] != 2.0 {
                npc.net_update = true;
            }
            npc.ai[1] = 2.0;
        }
    }
    if world.day_time {
        // `:33118-33122` — dawn: the fall and the horizontal decay.
        npc.vy += 0.3;
        npc.vx *= 0.9;
    } else if npc.ai[1] == 0.0 {
        // `:33123-33169` — the hover: 300 ticks, then the mode-1 scythe
        // charge (or the hover restarts). The aim hovers 200 px above the
        // player's centre; `num875` (6, or 8/10/12 by distance in mode 1)
        // rides the `/15` lerp.
        npc.ai[2] += 1.0;
        if npc.ai[2] >= 300.0 {
            if npc.ai[3] != 1.0 {
                npc.ai[1] = 0.0;
                npc.ai[2] = 0.0;
                npc.net_update = true;
            } else {
                npc.ai[1] = 1.0;
                npc.ai[2] = 0.0;
                target_closest(npc, world, player, 58);
                (pcx, pcy) = player.center();
                // `:33139` — the same kept `player13` re-read.
                #[allow(unused_assignments)]
                {
                    ptop = player.y;
                }
                npc.net_update = true;
            }
        }
        let num872 = pcx - cx;
        let num873 = pcy - 200.0 - cy;
        let mut num874 = (num872 * num872 + num873 * num873).sqrt();
        let mut num875 = 6.0f32;
        if npc.ai[3] == 1.0 {
            if num874 > 900.0 {
                num875 = 12.0;
            } else if num874 > 600.0 {
                num875 = 10.0;
            } else if num874 > 300.0 {
                num875 = 8.0;
            }
        }
        if num874 > 50.0 {
            num874 = num875 / num874;
            npc.vx = (npc.vx * 14.0 + num872 * num874) / 15.0;
            npc.vy = (npc.vy * 14.0 + num873 * num874) / 15.0;
        }
    } else if npc.ai[1] == 1.0 {
        // `:33170-33185` — the scythe charge: 600 ticks (or the mode
        // leaves 1), the 16-speed aim on the `/50` lerp.
        npc.ai[2] += 1.0;
        if npc.ai[2] >= 600.0 || npc.ai[3] != 1.0 {
            npc.ai[1] = 0.0;
            npc.ai[2] = 0.0;
            npc.net_update = true;
        }
        let num876 = pcx - cx;
        let num877 = pcy - cy;
        let num878 = 16.0 / (num876 * num876 + num877 * num877).sqrt();
        npc.vx = (npc.vx * 49.0 + num876 * num878) / 50.0;
        npc.vy = (npc.vy * 49.0 + num877 * num878) / 50.0;
    } else if npc.ai[1] == 2.0 {
        // `:33186-33195` — the flee: the slow sink, the 0.95 decays and
        // the 500-tick despawn window.
        npc.vy += 0.1;
        if npc.vy < 0.0 {
            npc.vy *= 0.95;
        }
        npc.vx *= 0.95;
        npc.encourage_despawn(500);
    }
    // `:33196` — the velocity tilt.
    npc.rotation = npc.vx * -0.02;
    // `NPC.cs:90495-90498` — the noTileCollide integration.
    npc.x += npc.vx;
    npc.y += npc.vy;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::TILE_PX;

    /// An air world with a solid floor at tile row 100 (the npc.rs tests'
    /// ground shape). The world is 200 tiles tall on purpose:
    /// `Collision.SolidCollision` clamps its sample rows to `maxTilesY - 40`
    /// (`Collision.cs:2712-2713` — the `Player::solid_collision` port), so
    /// in a 120-tall world row 100 would be invisible to the Wood's probe.
    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 100..120 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.day_time = false;
        w
    }

    fn night_player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 120.0 * TILE_PX;
        p.y = 95.0 * TILE_PX;
        p
    }

    fn wood() -> Npc {
        let mut n = Npc::new(
            NpcKind::MourningWood,
            60.0 * TILE_PX,
            60.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n
    }

    fn pumpking() -> Npc {
        let mut n = Npc::new(NpcKind::Pumpking, 60.0 * TILE_PX, 60.0 * TILE_PX, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// `:32792-32816` — the idle counts 300 ticks at full hp and then
    /// draws exactly ONE `Next(1, 3)` on the caller's stream (witness
    /// replay: the stream after the machine equals a fresh stream with
    /// the one hand-replayed draw).
    #[test]
    fn s1_f_13b2_wood_idle_picks_one_state_with_one_witnessed_draw() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = wood();
        let mut rng = UnifiedRandom::new(11);
        for _ in 0..299 {
            mourning_wood_step(&mut n, &w, &p, &mut rng);
            assert_eq!(n.ai[0], 0.0, "the idle holds under 300");
        }
        assert_eq!(
            rng,
            UnifiedRandom::new(11),
            "no draws before the 300th tick"
        );
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        let mut witness = UnifiedRandom::new(11);
        let pick = witness.next_range(1, 3);
        assert_eq!(n.ai[0], pick as f32, "the state is the drawn pick");
        assert!((1.0..=2.0).contains(&n.ai[0]), "full hp picks 1 or 2");
        assert_eq!(n.ai[1], 0.0, "the counter reset");
        assert_eq!(rng, witness, "exactly the one witnessed draw");
        assert!(n.net_update);
    }

    /// `:32795-32812` — under 25% hp the counter runs triple-time (100
    /// ticks reach 300) and the pick comes from `Next(3, 5)`.
    #[test]
    fn s1_f_13b2_wood_low_hp_triples_the_idle_and_picks_3_or_4() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = wood();
        n.hp = n.max_hp / 4 - 1;
        let mut rng = UnifiedRandom::new(5);
        for _ in 0..100 {
            mourning_wood_step(&mut n, &w, &p, &mut rng);
        }
        let mut witness = UnifiedRandom::new(5);
        let pick = witness.next_range(3, 5);
        assert_eq!(n.ai[0], pick as f32);
        assert!((3.0..=4.0).contains(&n.ai[0]));
        assert_eq!(rng, witness, "one Next(3,5) draw, nothing else");
    }

    /// `:32846-32865` — the stomp fires the flaming wood (325, 50 dmg)
    /// every 15 ticks from centre+30, aimed at the player's TOP-centre at
    /// speed 10 with the ±20×0.01 jitters — two witnessed draws per shot,
    /// in the C#'s SpeedX-then-SpeedY order.
    #[test]
    fn s1_f_13b2_wood_stomp_aims_at_the_player_top_with_witnessed_jitters() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = wood();
        n.ai[0] = 1.0;
        n.ai[1] = 14.0;
        let (ox, oy) = (n.center().0, n.center().1 + 30.0);
        let mut rng = UnifiedRandom::new(3);
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, x, y, sx, sy, dmg) = n.wants_hostile_shot[0];
        assert_eq!(kind, crate::projectile::FLAMING_WOOD);
        assert_eq!((x, y), (ox, oy));
        assert_eq!(dmg, 50);
        // The hand-replayed aim: the TOP-centre target, not the centre.
        let mut dx = p.x + crate::player::WIDTH * 0.5 - ox;
        let mut dy = p.y - oy;
        let s = 10.0 / (dx * dx + dy * dy).sqrt();
        dx *= s;
        dy *= s;
        let mut witness = UnifiedRandom::new(3);
        let jx = witness.next_range(-20, 21);
        let jy = witness.next_range(-20, 21);
        assert_eq!(sx, dx * (1.0 + jx as f32 * 0.01));
        assert_eq!(sy, dy * (1.0 + jy as f32 * 0.01));
        assert_eq!(rng, witness, "exactly the two jitter draws");
    }

    /// `:32860-32864` — the stomp ends at 120 ticks.
    #[test]
    fn s1_f_13b2_wood_stomp_ends_at_120() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = wood();
        n.ai[0] = 1.0;
        n.ai[1] = 119.0;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!((n.ai[0], n.ai[1]), (0.0, 0.0));
    }

    /// `:32893-32920` — the Greek-fire spin fires only on the 8th ticks
    /// strictly inside (60, 240); each shot draws FIVE rolls in the C#'s
    /// order: aim-x jitter, aim-y jitter, speed-x, speed-y, kind (326-328)
    /// — the `Type:` named argument sits LAST in the `NewProjectile` call
    /// (`:32913`), and C# evaluates named arguments in textual order.
    #[test]
    fn s1_f_13b2_wood_spin_cadence_and_five_draw_order() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = wood();
        n.ai[0] = 2.0;
        n.ai[1] = 63.0; // next tick 64: the first fire tick past 60
        let mut rng = UnifiedRandom::new(9);
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, _, _, _, _, dmg) = n.wants_hostile_shot[0];
        assert!((326..=328).contains(&kind));
        assert_eq!(dmg, 40);
        let mut witness = UnifiedRandom::new(9);
        let _dx = witness.next_range(-50, 51);
        let _dy = witness.next_range(50, 201);
        let _jx = witness.next_range(-30, 31);
        let _jy = witness.next_range(-30, 31);
        let k = witness.next_range(326, 329);
        assert_eq!(kind, k as u16, "the kind is the FIFTH draw");
        assert_eq!(rng, witness, "exactly five draws in the C# order");
        // Tick 65 is not a fire tick and draws nothing.
        let before = rng.clone();
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1);
        assert_eq!(rng, before, "no draws off the cadence");
        // The window's strict bounds: ai[1] = 240 does not fire.
        n.ai[1] = 239.0;
        let before = rng.clone();
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1, "240 is outside the window");
        assert_eq!(rng, before);
        // And the state ends at 300.
        n.ai[1] = 299.0;
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!((n.ai[0], n.ai[1]), (0.0, 0.0));
    }

    /// `:32922-32941`/`:32942-32969` — the beam (30-tick, 75 dmg, 0.001
    /// jitter) and the barrage (10-tick, 50 dmg, 0.005 jitter) both raise
    /// the chase speed to 4 and end at 120/240.
    #[test]
    fn s1_f_13b2_wood_beam_and_barrage_rows() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = wood();
        n.ai[0] = 3.0;
        n.ai[1] = 29.0;
        let mut rng = UnifiedRandom::new(1);
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1);
        assert_eq!(n.wants_hostile_shot[0].0, crate::projectile::FLAMING_WOOD);
        assert_eq!(n.wants_hostile_shot[0].5, 75);
        n.ai[1] = 119.0;
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!((n.ai[0], n.ai[1]), (0.0, 0.0));
        let mut n = wood();
        n.ai[0] = 4.0;
        n.ai[1] = 9.0;
        let mut rng = UnifiedRandom::new(2);
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, _, _, _, _, dmg) = n.wants_hostile_shot[0];
        assert!((326..=328).contains(&kind));
        assert_eq!(dmg, 50);
        n.ai[1] = 239.0;
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!((n.ai[0], n.ai[1]), (0.0, 0.0));
    }

    /// `:32970-32992` — above the player (|dx| < 50) the horizontal
    /// velocity damps ×0.9 and snaps to 0 inside ±0.1; otherwise the
    /// `(vx*20 + num836)/21` approach runs at the hp-band speed.
    #[test]
    fn s1_f_13b2_wood_horizontal_damp_and_hp_band_approach() {
        let w = ground_air();
        let mut p = night_player(&w);
        let mut n = wood();
        // Park the wood's centre exactly over the player's centre.
        p.x = n.center().0 - crate::player::WIDTH * 0.5;
        n.vx = 0.1;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vx, 0.0, "0.1*0.9 snaps to 0 inside ±0.1");
        // Far away, full hp: the 2/21 approach from rest.
        let mut n = wood();
        let p = night_player(&w);
        n.facing = 1;
        n.vx = 0.0;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vx, 2.0 / 21.0);
        // Under 50% hp the band is 4 — with the player LEFT of the wood
        // (the night retarget faces the player, `:77751-77760`).
        let mut n = wood();
        n.hp = n.max_hp / 2 - 1;
        let mut p = night_player(&w);
        p.x = n.center().0 - 400.0 - crate::player::WIDTH * 0.5;
        n.facing = -1;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vx, -4.0 / 21.0);
    }

    /// `:32993-33042` — the vertical hover: over the player +0.5; over
    /// solid ground the rise ramp (−0.025 then −0.2, cap −4); in open air
    /// the fall ramp (+0.025 then +0.5, cap 10).
    #[test]
    fn s1_f_13b2_wood_vertical_hover_ramps() {
        let w = ground_air();
        let p = night_player(&w);
        // Open air: the fall ramp from rest.
        let mut n = wood();
        n.vy = 0.0;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vy, 0.025);
        n.vy = 5.0;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vy, 5.5);
        n.vy = 9.8;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vy, 10.0, "the fall cap");
        // Just above the floor: the probe (80×20 at bottom−20) overlaps the
        // row-100 tiles (bottom at floor+4) and the rise ramp runs.
        let mut n = wood();
        n.y = 100.0 * TILE_PX - n.size().1 + 4.0;
        n.vy = 3.0;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(
            n.vy, -0.025,
            "the positive velocity zeroes, then the slow rise"
        );
        n.vy = -1.0;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vy, -1.2, "past -0.2 the fast rise");
        n.vy = -3.95;
        mourning_wood_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vy, -4.0, "the rise cap");
    }

    /// `:32787-32791` — dawn: `EncourageDespawn(10)` and the 8-speed
    /// approach; no attack-state draws run in the day arm.
    #[test]
    fn s1_f_13b2_wood_dawn_flees_at_8_and_arms_the_despawn() {
        let mut w = ground_air();
        w.day_time = true;
        let p = night_player(&w);
        let mut n = wood();
        n.facing = 1;
        let mut rng = UnifiedRandom::new(4);
        mourning_wood_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.despawn_timer, 10);
        assert_eq!(n.vx, 8.0 / 21.0);
        assert_eq!(rng, UnifiedRandom::new(4), "the day arm draws nothing");
    }

    /// `:33046-33055` — the Pumpking's frame cycle: localAI[0] wraps at 7
    /// ticks, localAI[1] cycles 0..4.
    #[test]
    fn s1_f_13b2_pumpking_frame_cycle() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = pumpking();
        for _ in 0..7 {
            pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        }
        assert_eq!((n.local_ai[0], n.local_ai[1]), (0.0, 1.0));
        for _ in 0..28 {
            pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        }
        assert_eq!(
            (n.local_ai[0], n.local_ai[1]),
            (0.0, 0.0),
            "the 5-frame wrap"
        );
    }

    /// `:33061-33069` — the attack-mode pick: at localAI[2] > 300 exactly
    /// one `Next(3)` draw sets ai[3] (witness replay).
    #[test]
    fn s1_f_13b2_pumpking_mode_pick_is_one_witnessed_next3() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = pumpking();
        n.ai[3] = 1.0; // off the lob arm: no other draws
        n.local_ai[2] = 300.0;
        let mut rng = UnifiedRandom::new(21);
        pumpking_step(&mut n, &w, &p, &mut rng);
        let mut witness = UnifiedRandom::new(21);
        let pick = witness.next_max(3);
        assert_eq!(n.ai[3], pick as f32);
        assert_eq!(n.local_ai[2], 0.0);
        assert_eq!(rng, witness, "exactly the one Next(3) draw");
        assert!(n.net_update);
    }

    /// `:33070-33085` — the mode-0 lob: every 30th tick past 30, from a
    /// non-solid origin, five draws (dx, dy, jx, jy, kind — `Type:` last,
    /// `:33083`); the dy rides `(playerTop − oy + Next(50,201)) * 0.2`.
    /// A SOLID origin tile suppresses the shot AND every draw.
    #[test]
    fn s1_f_13b2_pumpking_lob_draws_and_the_solid_origin_gate() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = pumpking();
        n.ai[0] = 1.0; // skip the init retarget
        n.ai[3] = 0.0;
        n.local_ai[2] = 59.0; // next tick 60: the fire tick
        let (ox, oy) = (n.center().0, n.center().1 + 30.0);
        let mut rng = UnifiedRandom::new(8);
        pumpking_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.wants_hostile_shot.len(), 1);
        let (kind, x, y, sx, sy, dmg) = n.wants_hostile_shot[0];
        assert!((326..=328).contains(&kind));
        assert_eq!((x, y), (ox, oy));
        assert_eq!(dmg, 40);
        let mut witness = UnifiedRandom::new(8);
        let mut dx = p.center().0 - ox + witness.next_range(-50, 51) as f32;
        let mut dy = (p.y - oy + witness.next_range(50, 201) as f32) * 0.2;
        let s = 5.0 / (dx * dx + dy * dy).sqrt();
        dx *= s;
        dy *= s;
        assert_eq!(sx, dx * (1.0 + witness.next_range(-30, 31) as f32 * 0.01));
        assert_eq!(sy, dy * (1.0 + witness.next_range(-30, 31) as f32 * 0.01));
        let k = witness.next_range(326, 329);
        assert_eq!(kind, k as u16, "the kind is the FIFTH draw");
        assert_eq!(rng, witness, "the five draws in the C# order");
        // The solid-origin gate: bury the origin tile.
        let mut w2 = ground_air();
        let (tx, ty) = ((ox as i32) / 16, (oy as i32) / 16);
        let i = w2.index(tx, ty);
        w2.tiles[i] = tr_core::tile::Tile {
            kind: 1,
            ..Default::default()
        };
        w2.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut n2 = pumpking();
        n2.ai[0] = 1.0;
        n2.local_ai[2] = 59.0;
        let mut rng2 = UnifiedRandom::new(8);
        pumpking_step(&mut n2, &w2, &p, &mut rng2);
        assert!(
            n2.wants_hostile_shot.is_empty(),
            "the solid origin holds the shot"
        );
        assert_eq!(rng2, UnifiedRandom::new(8), "and every draw with it");
    }

    /// `:33087-33104` + `:33123-33169` — the init arms ai[0]=1 and the
    /// hover lerps toward 200 px above the player at `num875 = 6`: from
    /// rest, vx = dx·(6/dist)/15 on the first tick.
    #[test]
    fn s1_f_13b2_pumpking_init_and_hover_lerp() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = pumpking();
        let (cx, cy) = n.center();
        let (pcx, pcy) = p.center();
        let dx = pcx - cx;
        let dy = pcy - 200.0 - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.ai[0], 1.0, "the init arms the hover");
        assert!(n.net_update);
        assert_eq!(n.vx, (0.0 * 14.0 + dx * (6.0 / dist)) / 15.0);
        assert_eq!(n.vy, (0.0 * 14.0 + dy * (6.0 / dist)) / 15.0);
        // The integration ran (noTileCollide).
        assert_eq!(n.center().0, cx + n.vx);
    }

    /// `:33147-33162` — in mode 1 the hover speed bands by distance:
    /// >900 → 12, >600 → 10, >300 → 8, else 6.
    #[test]
    fn s1_f_13b2_pumpking_mode1_speed_bands() {
        let w = ground_air();
        let p = night_player(&w);
        for (gap, want) in [
            (1000.0f32, 12.0f32),
            (700.0, 10.0),
            (400.0, 8.0),
            (200.0, 6.0),
        ] {
            let mut n = pumpking();
            n.ai[0] = 1.0;
            n.ai[3] = 1.0;
            // Park the boss so the hover aim (player centre − 200) is
            // exactly `gap` px to the right.
            n.x = p.center().0 - gap - n.size().0 * 0.5;
            n.y = p.center().1 - 200.0 - n.size().1 * 0.5;
            n.vx = 0.0;
            n.vy = 0.0;
            pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
            assert_eq!(n.vx, want / 15.0, "gap {gap} rides the {want} band");
            assert_eq!(n.vy, 0.0);
        }
    }

    /// `:33123-33141` — the hover's 300-tick gate: mode 1 enters the
    /// charge (ai[1]=1), any other mode restarts the hover counter.
    #[test]
    fn s1_f_13b2_pumpking_hover_gate_at_300() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = pumpking();
        n.ai[0] = 1.0;
        n.ai[3] = 1.0;
        n.ai[2] = 299.0;
        pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!((n.ai[1], n.ai[2]), (1.0, 0.0), "mode 1 charges");
        let mut n = pumpking();
        n.ai[0] = 1.0;
        n.ai[3] = 2.0;
        n.ai[2] = 299.0;
        pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!((n.ai[1], n.ai[2]), (0.0, 0.0), "mode 2 restarts the hover");
    }

    /// `:33170-33185` — the charge: the 16-speed aim on the /50 lerp; a
    /// mode change out of 1 (or 600 ticks) drops back to the hover.
    #[test]
    fn s1_f_13b2_pumpking_charge_lerp_and_exit() {
        let w = ground_air();
        let p = night_player(&w);
        let mut n = pumpking();
        n.ai[0] = 1.0;
        n.ai[1] = 1.0;
        n.ai[3] = 1.0;
        n.vx = 2.0;
        n.vy = -1.0;
        let (cx, cy) = n.center();
        let (pcx, pcy) = p.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let s = 16.0 / (dx * dx + dy * dy).sqrt();
        pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.vx, (2.0 * 49.0 + dx * s) / 50.0);
        assert_eq!(n.vy, (-1.0 * 49.0 + dy * s) / 50.0);
        // The mode exit.
        n.ai[3] = 0.0;
        pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!((n.ai[1], n.ai[2]), (0.0, 0.0));
    }

    /// `:33105-33117` + `:33186-33195` — a dead target flees: ai[1]=2,
    /// the 0.95 decays, the +0.1 sink and `EncourageDespawn(500)`.
    #[test]
    fn s1_f_13b2_pumpking_dead_target_flees() {
        let w = ground_air();
        let mut p = night_player(&w);
        p.dead = true;
        let mut n = pumpking();
        n.ai[0] = 1.0;
        n.vx = 4.0;
        n.vy = -2.0;
        pumpking_step(&mut n, &w, &p, &mut UnifiedRandom::new(0));
        assert_eq!(n.ai[1], 2.0);
        assert_eq!(n.vx, 4.0 * 0.95);
        assert_eq!(n.vy, (-2.0 + 0.1) * 0.95, "the negative rise decays");
        assert_eq!(n.despawn_timer, 500);
        // The rotation tilt (`:33196`).
        assert_eq!(n.rotation, n.vx * -0.02);
    }

    /// `:33118-33122` — dawn: vy += 0.3, vx *= 0.9, no state draws.
    #[test]
    fn s1_f_13b2_pumpking_dawn_fall() {
        let mut w = ground_air();
        w.day_time = true;
        let p = night_player(&w);
        let mut n = pumpking();
        n.ai[0] = 1.0;
        n.ai[3] = 1.0; // off the lob arm
        n.vx = 3.0;
        n.vy = 1.0;
        let mut rng = UnifiedRandom::new(6);
        pumpking_step(&mut n, &w, &p, &mut rng);
        assert_eq!(n.vx, 3.0 * 0.9);
        assert_eq!(n.vy, 1.3);
        assert_eq!(rng, UnifiedRandom::new(6), "the day arm draws nothing");
    }

    /// The wiring: the style axis claims 57/58 for these kinds through
    /// `Npc::update_rng` — the wood's idle pick and the Pumpking's mode
    /// pick land on the caller's stream exactly like the pure calls, and
    /// the Retinazer (aiStyle 30) never enters this module.
    #[test]
    fn s1_f_13b2_the_style_axis_routes_the_bosses() {
        assert_eq!(NpcKind::MourningWood.ai_style(), 57);
        assert_eq!(NpcKind::Pumpking.ai_style(), 58);
        let w = ground_air();
        let mut p = night_player(&w);
        // The wood through update_rng: 300 idle ticks, one witnessed draw
        // (the contact pass is out of range — the player is 60 tiles away).
        let mut n = wood();
        let mut rng = UnifiedRandom::new(11);
        for _ in 0..300 {
            n.update_rng(&w, &mut p, 1.0 / 60.0, &mut rng);
        }
        let mut witness = UnifiedRandom::new(11);
        let pick = witness.next_range(1, 3);
        assert_eq!(n.ai[0], pick as f32, "the axis runs the named machine");
        assert_eq!(rng, witness, "the UpdateNPCs stream order holds");
        // The Pumpking through update_rng: the init + the mode pick.
        let mut n = pumpking();
        n.ai[3] = 1.0;
        n.local_ai[2] = 300.0;
        let mut rng = UnifiedRandom::new(21);
        n.update_rng(&w, &mut p, 1.0 / 60.0, &mut rng);
        let mut witness = UnifiedRandom::new(21);
        let pick = witness.next_max(3);
        assert_eq!(n.ai[3], pick as f32);
        assert_eq!(rng, witness);
        // The Retinazer keeps its own routing (style 30, the kind match).
        assert_eq!(NpcKind::Retinazer.ai_style(), 30);
    }
}
