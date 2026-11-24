//! Projectile simulation: the fixed 60 Hz pass over the 1000-slot pool
//! (`Projectile.Update`, see `projectile.rs` for the source map).

use glam::Vec2;

use crate::app::App;
use crate::projectile::{FlailInputs, SHURIKEN, YoyoInputs};

/// Fixed 60 Hz pool stepping. Damage lands through `Npc::hurt`, so the
/// knockback chain is shared with melee; the immunity bookkeeping is the
/// pool's (`localNPCImmunity` / `perIDStaticNPCImmunity` / `NPC.immune`).
pub(crate) fn update_projectiles(app: &mut App, dt: f32) {
    // `TR_THROW`: the projectile-sprite fixture — one shot of the pinned kind
    // from the player's centre, through the same `spawn` the items use.
    // One shot only: the fixture re-arms when the pool empties.
    if let Some(kind) = crate::env::throw_kind() {
        let empty = app.projectiles.slots().iter().all(|p| !p.active);
        if empty {
            if let Some(p) = &app.player {
                let (cx, cy) = p.center();
                if kind == crate::projectile::LIGHTS_BANE {
                    // The slash's scale rides `ai[0]` (`AI_188_LightsBane`,
                    // `Projectile.cs:42240`) — the fixture spawns it through
                    // the SetDefaults row at scale 1 (S1-G-3's blend-batch
                    // screenshot).
                    app.projectiles.spawn_with_data_owned(
                        kind,
                        0,
                        cx - 3.0,
                        cy - 3.0,
                        3.0,
                        0.0,
                        10,
                        4.0,
                        1.0,
                        0.0,
                    );
                } else {
                    app.projectiles.spawn(
                        kind,
                        0,
                        Vec2::new(cx - 3.0, cy - 3.0),
                        Vec2::new(3.0, 0.0),
                        10,
                        4.0,
                        false,
                    );
                }
            }
        }
    }
    if app.dev_throw {
        if let Some(p) = &app.player {
            let (cx, cy) = p.center();
            app.projectiles.spawn(
                SHURIKEN,
                0,
                Vec2::new(cx - 3.0, cy - 3.0),
                Vec2::new(11.0, 0.0),
                10,
                12.0,
                false,
            );
        }
        app.dev_throw = false;
    }
    if app.summon_shot {
        if let Some(p) = &mut app.player {
            let (cx, cy) = p.center();
            // Phase 3's pool block is live now: the minion books against the
            // summon budget, so the debug fixture grants the slots the pair
            // needs (`maxMinions = 1` base would self-kill the Hornet on its
            // first pool tick — the cap semantics themselves are Phase 3's
            // tests; this fixture only wants both sprites on screen).
            p.max_minions = 3;
            // The DW-1.4 pair (M4-PROJ-8 Phase 1): a Hornet (373) + a Frost
            // Hydra (308) ride the generic active-slot draw pass
            // (`frame.rs`'s `p.draw` loop — the plan's Decision Log: presence
            // via the placeholder draw, not per-kind art). Damage 0 keeps the
            // PVE walk a no-op; the unported aiStyles 62/53 dispatch no arm,
            // so both slots hold position for the capture.
            // The pair anchors on the player's centre, clamped strictly
            // inside the world bounds (the guard's test is `x + w >=
            // rightWorld`, so a slot flush on the edge is out): Phase 3's
            // out-of-world guard (`Projectile.cs:16683-16703`, rightWorld /
            // bottomWorld = maxTiles * 16) would deactivate such a slot on
            // the very next pool tick, and the front-flow worlds run as
            // small as 8x8 tiles. On a real world the clamp is a no-op — the
            // DW-1.4 screenshot positions are unchanged.
            let (ww, wh) = (
                app.world.width as f32 * 16.0,
                app.world.height as f32 * 16.0,
            );
            let hx = (cx - 24.0).clamp(0.0, (ww - 24.0 - 1.0).max(0.0));
            let hy = (cy - 48.0).clamp(0.0, (wh - 26.0 - 1.0).max(0.0));
            let gx = (cx + 24.0).clamp(0.0, (ww - 80.0 - 1.0).max(0.0));
            let gy = (cy - 48.0).clamp(0.0, (wh - 74.0 - 1.0).max(0.0));
            app.projectiles.spawn_with_data_ai_owned(
                crate::projectile::HORNET,
                0,
                hx,
                hy,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
            app.projectiles.spawn_with_data_ai_owned(
                crate::projectile::FROST_HYDRA,
                0,
                gx,
                gy,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
        }
        app.summon_shot = false;
    }
    // The boomerangs' return seek reads the owner's center (`AI_003`'s
    // `Main.player[owner]` inputs); the whips anchor their curve on the same
    // center (the `GetArmPosition` stand-in) and aim the Moon Lord arms
    // along `swing_aim` (`AI_165_Whip`'s mouse aim, `:47709`).
    let owner = app.player.as_ref().map(|p| {
        let (x, y) = p.center();
        (x, y)
    });
    // `AI_099_2_Yoyos`'s `Main.player[owner]` reads: the aim world point
    // (`Main.ReverseGravitySupport(Main.MouseScreen) + Main.screenPosition`,
    // `Projectile.cs:67928` — the same mouse-world math editing.rs:26 uses),
    // the held-button channel state (`Player.channel`, the release arm
    // `:67920-67925`), the CCed gate (`:67920`'s `|| CCed`), the owner's
    // death (`:67824`) and `meleeSpeed` (the lifetime divisor `:67750` + the
    // range/top-speed division `:67886-67887` — S1-P-i09). The channel reads
    // the LATCHED `p.channel` (armed at the use start, cleared on
    // release/CC/kill/death — S1-P-i23), not a per-tick re-derive from the
    // button: a tracked-shot death must be able to drop the channel mid-hold
    // (`TryCancelChannel` `Player.cs:52975`).
    let (sw, sh) = app.screen_size();
    // The `controlUseItem` zeroing gates (`Player.cs:42804-42807`'s
    // `channel = false` on the cleared input): the inventory
    // (`Main.cs:17659-17663`'s `player.mouseInterface`/inventory arm), the
    // fullscreen map (`Player.cs:24798-24807`) and the minimap hover
    // (`MinimapFrame.cs:164`) all clear the use input, so the channel
    // drops behind them exactly like `update_editing`'s own gates
    // (S1-P-i09 — the old snapshot ignored them).
    let use_input_live = !app.inventory_open
        && !app.map.as_ref().is_some_and(|m| m.open)
        && app.minimap_hover.is_none();
    let inputs = YoyoInputs {
        aim: app.player.as_ref().map(|_| {
            (
                app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
            )
        }),
        // S1-P-i23's latched channel AND S1-P-i09's use-input gates: the
        // inventory/map/minimap clears drop the use input, so the channel
        // drops behind them even before `update_editing` clears the latch.
        channel: app.player.as_ref().is_some_and(|p| p.channel) && use_input_live,
        player_dead: app.player.as_ref().is_some_and(|p| p.dead),
        player_cced: app.player.as_ref().is_some_and(|p| p.cced()),
        melee_speed: app
            .player
            .as_ref()
            .map_or(1.0, |p| p.melee_speed(app.inventory[app.selected].kind)),
        // S1-E-21h d75 — the drill hum's `inventory[selectedItem].pick > 0`
        // read (`Projectile.cs:25512`), snapshotted off the held ItemDef.
        held_pick_positive: crate::registry::item(app.inventory[app.selected].kind)
            .is_some_and(|it| it.pick > 0),
        // W4-d10 — `AI_099_1_Counterweights`'s channel push (`:67521`):
        // the frame-head `UpdateMainMouse` snapshot (`PlayerInput.cs:
        // 911-917`), already in the world-update zoom basis
        // (`SetZoom_MouseInWorld`, `:2528-2542`). Drained per projectile
        // tick — every sub-step reads the same per-tick delta, exactly
        // like the C# reads the same `Main.mouseX/lastMouseX` pair.
        mouse_delta: (app.mouse_tick_delta.x, app.mouse_tick_delta.y),
        // W4-d10 — the yoyo-string accessory grants (`Player.cs:14710-
        // 14754`, the `ApplyEquipFunctional` rows; `ResetEffects` clears
        // per tick `:18435`/`:18439`). The yoyo arm's lifetime/range terms
        // and the counterweight's radius/ghost split read them.
        yoyo_string: app.player.as_ref().is_some_and(|p| p.yoyo_string),
        magic_string: app.player.as_ref().is_some_and(|p| p.magic_string),
    };
    // The whips anchor their curve on the player center and aim the Moon
    // Lord arms along the same aim point (`AI_165_Whip`'s mouse aim, `:47709`)
    // — `inputs.aim` minus the center, so the two systems share one read.
    let aim = inputs
        .aim
        .zip(owner)
        .map(|((ax, ay), (ox, oy))| (Vec2::new(ax - ox, ay - oy)).normalize_or_zero())
        .filter(|d| *d != Vec2::ZERO);
    // `AI_015_Flails`'s `Main.player[owner]` reads (`Projectile.cs:43726-
    // 44169`): the same aim/channel/death snapshot plus the chain anchor
    // (`MountedCenter`, `:3335`), the kill-gate reads (`CCed`), the throw
    // add and the state-3 stand-still read (`player.velocity`), the hold
    // orbit's `player.direction`, the meleeSpeed scaling and the local
    // owner gate on the family's spawns.
    let flail_inputs = app.player.as_ref().map_or_else(
        || FlailInputs { player_dead: true, ..FlailInputs::default() },
        |p| {
            let (cx, cy) = p.center();
            let (mx, my) = p.mounted_center();
            // `Main.LocalPlayer.inventory[selectedItem].shoot` (`:43850`) —
            // the dual-flail flag5's selected-item gate.
            let held = app.inventory[app.selected];
            let selected_shoot = if held.count > 0 {
                crate::registry::projectile(held.kind)
            } else {
                None
            };
            FlailInputs {
                aim: inputs.aim,
                channel: inputs.channel,
                // `Player.controlUseItem` — the raw use button (the port's
                // `mine_down`), NOT the channel state: the flight/retract/
                // spin re-grab arms read the button even when `channel` is
                // false (`Projectile.cs:43905/43942/43959/44035`).
                control_use_item: app.mine_down,
                map_fullscreen: app.map.as_ref().is_some_and(|m| m.open),
                player_dead: false,
                player_cced: p.cced(),
                mounted_center: Some((mx, my)),
                center: Some((cx, cy)),
                player_vel: (p.vx, p.vy),
                player_dir: p.facing,
                // `player.meleeSpeed` as the flail reads it — the inverted,
                // held-item-scaled multiplier (`Player.cs:25912` +
                // `CapAttackSpeeds` `:29021-29039`), keyed on the owner's
                // selected item exactly like `BonusMeleeSpeedMultiplier
                // [inventory[selectedItem].type]`.
                melee_speed: p.melee_speed(app.inventory[app.selected].kind),
                local_owner: app.local_id as u8,
                selected_shoot,
            }
        },
    );
    // The closure's disjoint field captures keep the step borrow
    // (`projectile_step`) apart from the pool/world/npcs/player set. The
    // yoyo rolls (`Projectile.cs:67746`'s jitter, `:67886`'s beam rolls,
    // `:68041`'s penetrate roll) draw `Main.rand` under the
    // `UpdateProjectiles` swap (`Main.cs:17775-17777`) — the pass binds its
    // own named stream. Only this machine's projectiles simulate here — a
    // peer's shots ride the `27` wire (`Projectile.cs:17210`'s owner gate),
    // and the server-owned hostile shots (owner 255) only on the host /
    // single-player (`spawn_hostile`).
    let (local_owner, hostile_local) = (app.local_id as u8, !app.is_client());
    let rng = app.rngs.stream(crate::app::RngKey::UpdateProjectiles);
    // S1-P-i06 — the spear family's owner snapshot (`AI_019_Spears`'s
    // `Main.player[owner]` reads, `Projectile.cs:45859-45868`). The C# reads
    // the live `itemAnimation`/`itemAnimationMax` integers; the port's swing
    // timers run in seconds, so the counts derive from `app.swing` /
    // `app.swing_max` (the `ticks_of` convention, `world_sim/editing.rs`).
    // The anchor is `RotatedRelativePoint(MountedCenter)` (`:45860`).
    let spear_owner = app.player.as_ref().map(|p| {
        let (mcx, mcy) = p.mounted_center();
        let anchor = p.rotated_relative_point(Vec2::new(mcx, mcy), 0);
        crate::projectile_ai_spear::SpearOwner {
            anchor,
            direction: p.facing,
            item_animation: (app.swing * 60.0).round() as i32,
            item_animation_max: (app.swing_max * 60.0).round() as i32,
            channel: p.channel,
            // i12's held-kind form (`Player.cs:25912` reads
            // `BonusMeleeSpeedMultiplier[selectedItem.type]`).
            melee_speed: p.melee_speed(app.inventory[app.selected].kind),
            wet: p.wet,
            player_vel: Vec2::new(p.vx, p.vy),
            player_pos: Vec2::new(p.x, p.y),
            player_w: crate::player::WIDTH,
            player_h: p.height,
        }
    });
    app.projectiles.set_spear_owner(spear_owner);
    // W4-d9 — the aiStyle-90 lantern's owner snapshot
    // (`Projectile.cs:32389-32416`'s `Main.player[owner]` reads; the scan
    // gate's `Main.myPlayer` centre folds onto the same owner — the port
    // simulates only the local owner's slots single-player). The screen
    // width rides `app.screen_size()` (the `Main.screenWidth` stand-in).
    let lantern_owner = app.player.as_ref().map(|p| {
        let (cx, cy) = p.center();
        crate::projectile_ai_style90::LanternOwner {
            center: Vec2::new(cx, cy),
            direction: p.facing,
            dead: p.dead,
            magic_lantern: p.magic_lantern,
        }
    });
    app.projectiles.set_lantern_owner(lantern_owner);
    app.projectiles.set_screen_width(sw as i32);

    // S1-P-i35 — the bobber arm's per-tick player/world snapshot (`AI_061` +
    // `FishingCheck`'s `Main.player[owner]` reads, `Projectile.cs:53658+`).
    // Built from the live player/world/scene state before the pass (the
    // `set_disco` precedent); `None` when no player exists.
    let bobber_inputs = app.player.as_ref().map(|p| {
        let held = app.inventory[app.selected];
        let held_def = crate::registry::get(held.kind);
        let (_, mounted_y) = p.mounted_center();
        let cond = p.fishing_conditions(
            &app.inventory,
            &app.equip.coins,
            &app.equip.ammo,
            app.selected,
            &app.world,
            app.rain.is_raining(),
            app.rain.cloud_bg(),
        );
        let quest_item = crate::angler::QUEST_POOL
            .get(app.world.angler_quest as usize)
            .copied()
            .unwrap_or(0) as i32;
        let player_has_quest_item = quest_item > 0
            && app.inventory.iter().any(|sl| sl.count > 0 && sl.kind as i32 == quest_item);
        let m = &app.scene_metrics;
        crate::projectile_bobber::BobberInputs {
            held_pole_power: held_def.map_or(0, |d| d.fishing_pole),
            held_shoot: held_def.and_then(|d| d.projectile).unwrap_or(0),
            cced: p.cced(),
            no_items: p.no_items,
            pulley: p.pulley,
            dead: p.dead,
            final_level: cond.4,
            bait_type: cond.3,
            pole_type: cond.1,
            player_wet: p.wet,
            player_mounted_y: mounted_y,
            luck: p.luck(),
            crate_potion: p.crate_potion,
            sonar_potion: p.sonar_potion,
            acc_lava_fishing: p.acc_lava_fishing,
            not_the_bees: app.world.seeds.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES),
            remix: app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
            blood_moon: app.world.blood_moon,
            day_time: app.world.day_time,
            hard_mode: app.world.hard_mode,
            unlocked_slime_red_spawn: app.world.unlocked_slime_red_spawn,
            duke_alive: app.npcs.iter().any(|n| n.kind == crate::npc::NpcKind::DukeFishron && n.hp > 0),
            quest_item,
            player_has_quest_item,
            angler_alive: app.npcs.iter().any(|n| n.kind == crate::npc::NpcKind::Angler && n.hp > 0),
            angler_quest_finished: app.world.angler_quest_finished,
            zone_corrupt: m.zone_corrupt,
            zone_crimson: m.zone_crimson,
            zone_jungle: m.zone_jungle,
            zone_snow: m.zone_snow,
            zone_dungeon: m.zone_dungeon,
            zone_desert: m.zone_desert,
            zone_hallow: m.zone_hallow,
            // `ZoneGlowshroom` has no port metric (the editing DropCtx
            // precedent, false — ANALYSIS).
            zone_glowshroom: false,
            zone_beach: m.zone_beach,
            downed_boss3: app.world.downed_skeletron,
            combat_book_was_used: app.world.combat_book_was_used,
        }
    });
    app.projectiles.set_bobber_inputs(bobber_inputs);
    // W4-d7 F3 — the Companion Cube's darkness read (`Projectile.cs:49527-
    // 49528`): the two `Lighting.GetColor` samples (the cube's centre tile +
    // the owner's centre tile) as `Color.ToVector3()` lengths. `GetColor`
    // (`Lighting.cs:216-242`) byte-quantizes `(int)(engineColor * GlobalBrightness
    // * 255)` clamped to 255, then `ToVector3` divides by 255 — so the length is
    // the brightness-scaled, byte-truncated colour. GlobalBrightness is 1.2
    // (`Lighting.cs:91`), dropped to 1.0 by the Darkness debuff's `blind` flag
    // (`:114`). Sampled from the live light map before the pass (the `set_disco`
    // precedent); only when a cube is owned.
    let cube_light = app.player.as_ref().and_then(|p| {
        let cube = app
            .projectiles
            .slots()
            .iter()
            .find(|s| s.active && s.kind == crate::projectile::COMPANION_CUBE && s.owner == 0)?;
        let brightness = if p.blind { 1.0 } else { tr_core::lighting::GLOBAL_BRIGHTNESS };
        // `Lighting.cs:224` precomputes `num = GlobalBrightness * 255f` ONCE,
        // then `(int)(color.X * num)` — the multiply order matters for the
        // byte truncation, so fold the scale first (not `v * b * 255`).
        let scale = brightness * 255.0;
        let len = |wx: f32, wy: f32| {
            let c = app.light.get((wx / 16.0) as i32, (wy / 16.0) as i32);
            let byte = |v: f32| ((v * scale) as i32).min(255) as f32 / 255.0;
            let (r, g, b) = (byte(c[0]), byte(c[1]), byte(c[2]));
            (r * r + g * g + b * b).sqrt()
        };
        let (pcx, pcy) = p.center();
        Some((
            len(cube.pos.x + cube.w * 0.5, cube.pos.y + cube.h * 0.5),
            len(pcx, pcy),
        ))
    });
    app.projectiles.set_cube_light(cube_light);
    // W4-d7 F3 — the Storm Tigers' special-attack gem count
    // (`ownedProjectileCounts[831]`, `Projectile.cs:49039`) + the
    // `Main.timeForVisualEffects` draw clock (the state-5 cosine bob,
    // `:49683`; the `anim_time * 60 % 216000` form, `frame/draw.rs:374`).
    let gem_counts = app.player.as_ref().map_or((0, 0), |p| {
        (p.owned_projectile_counts[831], p.owned_projectile_counts[970])
    });
    app.projectiles.set_tiger_gem_counts(gem_counts);
    app.projectiles.set_time_for_visuals((app.anim_time * 60.0) % 216000.0);
    // `Main.DiscoR/G/B` as the aiStyle-123 crystal's shard-hue roll reads
    // it (`Projectile.cs:37262`); the clock's `advance_disco` owns the cycle.
    let disco = app.disco;
    app.projectiles.set_disco(disco);
    // `Player.SceneMetrics.NPCBannerBuff` as the hostile shots' `Damage_EVP`
    // banner arm reads it (`Projectile.cs:14832` → `Player.cs:44197`); the
    // per-tick scene-scan snapshot (the `set_disco` precedent). W4-g4 P1
    // (the W4-d22 reconcile fold).
    app.projectiles
        .set_npc_banner_buff(app.scene_metrics.npc_banner_buff.clone());
    // W4-d1 — `Main.infectedSeed` as the PurificationPowder 10's spawn-time
    // box growth reads it (`Projectile.cs:907-911`): the world's
    // `SecretSeed.worldIsInfected` micro-seed flag (`WorldGen.cs:559-562`).
    app.projectiles.set_infected_seed(
        app.world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::WORLD_IS_INFECTED),
    );
    // W4-d3 — `Main.tileFrame[665]` as the aiStyle-25 1014 LifeCrystalBoulder's
    // frame pin reads it (`Projectile.cs:26028`); the tile animator owns the
    // row (`app.tile_anim`, S1-G-1f), the pool only reads it (the `set_disco`
    // pattern).
    let frame_665 = app.tile_anim.tile_frame[665];
    app.projectiles.set_tile_frame_665(frame_665);
    // W4-c19 round 3 — `Main.mouseTextColor` as the 590 TruffleSpore's flicker
    // reads it (`Projectile.cs:34221`); the draw-pass pulse owns the value
    // (the `set_disco` precedent — the step reads what the last draw left).
    app.projectiles.set_mouse_text_color(app.mouse_text_color);
    // S1-P-i55 — the golf context (`Main.LocalGolfState` + `Main.MouseWorld`
    // + the mouse-right pair + the player snapshot). The C# reads the live
    // state mid-walk; the port clones it into the pool for the tick and
    // writes the resolved clone back after (the strikes' RecordSwing and
    // the cup's scoring run inside the arms). `mouseRightRelease` lags one
    // frame in the C# (`Main.cs:62458-62465` runs after Draw): the pair
    // `mouseRight && mouseRightRelease` is therefore the PRESS tick — the
    // port's `place_down && !place_prev_down` is the same edge.
    let golf_ctx = app.player.as_ref().map(|p| {
        let (cx, cy) = p.center();
        let (bx, by) = p.bottom();
        let held = app
            .inventory
            .get(app.selected)
            .filter(|s| s.count > 0)
            .map(|s| s.kind)
            .unwrap_or(0);
        let use_animation = crate::registry::get(held).map_or(0, |d| d.use_animation as i32);
        crate::projectile_ai_golf::GolfContext {
            golf_state: app.golf_state.clone(),
            mouse_world: Vec2::new(
                app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
            ),
            mouse_right: app.place_down,
            // The C#'s `mouseRightRelease` lags one frame (the flag
            // refresh runs after Draw, `Main.cs:62458-62465`), so the
            // pair's `true && true` is the PRESS tick. The port's
            // `place_prev_down` is last frame's button state — the
            // release-lag stand-in.
            mouse_right_release: !app.place_prev_down,
            player: crate::projectile_ai_golf::GolfClubPlayer {
                // `player.active` is the slot-in-use flag, NOT liveness —
                // a dead player's helper keeps charging (the C# kill gate
                // `:52420` reads `active` only). The context exists only
                // when the player is present, so the flag is true.
                active: true,
                center: Vec2::new(cx, cy),
                bottom: Vec2::new(bx, by),
                direction: p.facing,
                channel: p.channel,
                use_animation,
                held_item_type: held as i32,
            },
            player_name: app.player_name.clone(),
            golfer_score: p.golfer_score,
        }
    });
    if let Some(ctx) = golf_ctx {
        app.projectiles.set_golf_context(ctx);
    }
    // W4-d3 — the boulder dust/gore lane's live seam (the W4-g64/g16 shapes):
    // the frame's spawn ctx (`Main.screenPosition/width/height` + the gfx
    // budget) captured BEFORE the step closure's field borrows, and the App's
    // gore pool moved in for the step (the `golf_ctx` take/replace precedent).
    let dust_spawn_ctx = app.spawn_ctx();
    let gore_pool = std::mem::replace(&mut app.gore, tr_core::gore::GorePool::new());
    app.projectiles.set_gore_pool(Some(gore_pool));
    // W4-g63 — the trail-dust lane's live seam: the frame's spawn ctx
    // (`Main.screenPosition/width/height` + the gfx budget) captured
    // BEFORE the step closure's `&mut app.world`/`&mut app.dust` field
    // borrows (the `world_sim/npcs.rs` `dust_spawn_ctx` convention).
    let dust_spawn_ctx = app.spawn_ctx();
    app.projectile_step.tick(dt.min(0.25), |_| {
        app.projectiles.update_dust(
            &mut app.world,
            &mut app.npcs,
            owner,
            aim,
            app.player.as_mut(),
            inputs,
            flail_inputs,
            rng,
            local_owner,
            hostile_local,
            &mut app.dust,
            &dust_spawn_ctx,
        );
    });
    // W4-d3 — the gore pool writeback: the boulder Kill arms spawned into it.
    if let Some(pool) = app.projectiles.take_gore_pool() {
        app.gore = pool;
    }
    app.projectile_step.cap_backlog();
    // W4-d21 — the digtoise dig drain: the aiStyle-204 arm queued the dig
    // cells (the hit-tile book + the best-pickaxe read live on the App);
    // resolve them on the same `UpdateProjectiles` stream the C#'s PickTile
    // runs under (`Main.cs:17775-17777`).
    crate::world_sim::editing::drain_digtoise_digs(app);
    // S1-P-i55 — the golf context writeback: the arms mutated the clone
    // (RecordSwing / the cup scoring); the resolved state is authoritative
    // for the tick. The `golfer_score` accumulator rides the player.
    if let Some(ctx) = app.projectiles.take_golf_context() {
        app.golf_state = ctx.golf_state;
        if let Some(p) = app.player.as_mut() {
            p.golfer_score = ctx.golfer_score;
        }

    }
    // S1-P-i06 — the lances' channel-hold write-back (`SetDummyItemTime(third)`,
    // `Projectile.cs:45941` → `Player.cs:4432-4441`: `itemAnimation = itemTime =
    // frames`, `itemTimeMax = frames + 1`). The arm queued the tick count; the
    // port re-arms the swing/item timers so the hold persists while the button
    // is down (the `FlailTick.facing` write-back precedent). `app.swing_max`
    // stays the item's animation length (the C# `SetDummyItemTime` does not
    // touch `itemAnimationMax`).
    if let Some(frames) = app.projectiles.take_spear_hold() {
        let secs = frames as f32 / 60.0;
        app.swing = secs;
        app.item_time = secs;
    }
    // S1-C-7 wave B — the hostile-shot lane's hurt modifiers (`Player.Hurt`
    // `:38396-38517` via `Projectile.Damage` `Projectile.cs:14868`): the
    // accepted shot queued its retaliate request and possibly a dismount-on-
    // hit (the golf-ball returns 877/878/879); the C# rolls inline under the
    // `UpdateProjectiles` swap (`Main.cs:17775`), the port drains at the pass
    // tail on the same stream (the S1-E-21c batch convention).
    if let Some(p) = app.player.as_mut() {
        crate::hurt_modifiers::drain_dismount(p, &app.world);
    }
    // W4-f50 — the player hurt combat text (`Player.cs:38382`): the C# draws
    // it BEFORE the retaliate block (`:38401`), so the drain spawns it ahead
    // of the retaliate rolls on the `UpdateProjectiles` stream.
    if let Some((amount, crit)) = app.player.as_mut().and_then(|p| p.take_hurt_text()) {
        let text = amount.to_string();
        let size = match app.lookups.combat_text_font.as_ref() {
            Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
            None => [text.chars().count() as f32 * 8.0, 10.0],
        };
        let p = app.player.as_mut().unwrap();
        let location = crate::combat_text::Rect {
            x: p.x,
            y: p.y,
            width: p.width,
            height: p.height,
        };
        let color = crate::combat_text::damage_text_color(crit, true, false);
        crate::combat_text::spawn_amount(
            &mut app.combat_texts,
            location,
            color,
            amount,
            crit,
            false,
            size,
            1.0,
            rng,
        );
    }
    if let Some(req) = app.player.as_mut().and_then(|p| p.take_retaliate()) {
        let expert = app.player.as_ref().is_some_and(|p| p.buffs.expert_mode);
        crate::hurt_modifiers::drain_retaliate(
            app.player.as_mut().unwrap(),
            &mut app.projectiles,
            &mut app.npcs,
            req,
            app.local_id as u8,
            expert,
            rng,
        );
    }
    // W4-f50 — the Solar Flare hurt blast (`Player.cs:38316-38318`): the
    // accepted hostile shot consumed a shield charge and queued the 608
    // blast; the C# spawns + Kills it inline under the `UpdateProjectiles`
    // swap (`Main.cs:17775`), the port drains at the pass tail on the same
    // stream (the S1-E-21c batch convention).
    if let Some((center, damage)) = app.player.as_mut().and_then(|p| p.take_solar_blast()) {
        let dolls = app.player.as_ref().map_or((false, false), |p| p.doll_flags());
        let luck = app.player.as_ref().map_or(0.0, |p| p.luck());
        let blast_ctx = app.spawn_ctx();
        app.projectiles.solar_flare_blast(
            center,
            damage,
            app.local_id as u8,
            &mut app.npcs,
            dolls,
            rng,
            luck,
            &mut app.dust,
            &blast_ctx,
            &app.world,
            &mut app.gore,
        );
    }
    // W4-f50 — the hit-dust loop + football + golf-club hurt drains
    // (`Player.cs:38527-38553` / `:38255-38280`): the accepted hostile shot
    // queued them; the port drains at the pass tail on the `UpdateProjectiles`
    // stream (the S1-E-21c batch convention).
    if let Some(p) = app.player.as_mut() {
        let ctx = app.spawn_ctx();
        crate::hurt_modifiers::drain_hurt_dust(
            p,
            &mut app.dust,
            &ctx,
            &app.world,
            rng,
        );
    }
    if let Some(p) = app.player.as_mut() {
        crate::hurt_modifiers::drain_football(
            p,
            &mut app.projectiles,
            &mut app.inventory,
            &mut app.mouse_item,
            app.local_id as u8,
        );
    }
    if let Some(p) = app.player.as_mut() {
        let held = app.inventory[app.selected].kind;
        crate::hurt_modifiers::drain_golf_club_kill(
            p,
            &mut app.projectiles,
            held,
            app.local_id as u8,
        );
    }
    // W4-c19 round 3 — the rand-gated AI-arm light intents
    // (`Projectile.cs:30378` typhoon / `:34226` spore / `:47846`/`:48011`
    // whips): the arms took their gate draws on the `UpdateProjectiles`
    // stream mid-walk and queued the lights in slot + sub-step order; the
    // drain appends them into the ONE `EntityLights` pair here (the
    // `item_lights.rs` named-stream precedent — the e6-rule exception, the
    // draw-pass feed stays pure). No RNG: the append is stream-neutral.
    for (x, y, rgb) in app.projectiles.drain_ai_lights() {
        app.entity_lights.add(x, y, rgb);
    }
    // S1-E-21 wave C — the landed strikes' `NPC.HitSound` plays
    // (`NPC.cs:81544-81553` via `Projectile.Damage` `Projectile.cs:13649`):
    // the case-3 volume arms + the `Main.rand.Next(-10, 11) * 0.01` pitch
    // draw ride the `UpdateProjectiles` stream (`Main.cs:17775`), only
    // after each play's attenuation gate passed. The C# draws mid-walk;
    // the port drains at the pass tail (ANALYSIS, S1-E-21c).
    for (cue, x, y) in app.projectiles.drain_npc_hit_sounds() {
        app.audio.play_npc_hit_sound(cue, x, y, rng);
    }
    // W4-f32 — the projectile-side Lucky Coin proc's drops
    // (`Projectile.cs:13685-13697`): the six rolls (the Next(5) gate, the
    // denomination pair, the DISCARDED Next(1, 11), the velocity pair) were
    // drawn inline mid-walk on the `UpdateProjectiles` stream; the pool
    // queued the fully-rolled drop (it cannot borrow app.items). The spawn
    // itself draws NOTHING — the explicit velocity skips `Item.NewItem`'s
    // spread draws (`Item.cs:49738-49740`) and prefix 0 draws nothing, so
    // the pass-tail placement is stream-exact (the drain convention).
    // Stack 1 (the C# discards its stack roll), `WorldItem.Center = center`
    // centres the 14x14 box on the NPC (`Item.cs:49736`).
    for (coin, ncx, ncy, vx, vy) in app.projectiles.drain_lucky_coin_drops() {
        let drop = crate::item::ItemDrop::new(
            coin,
            1,
            0,
            ncx - crate::item::SIZE * 0.5,
            ncy - crate::item::SIZE * 0.5,
            vx,
            vy,
            0.0,
        );
        app.items.push(drop);
    }
    // S1-E-21h — the projectile-family cue plays (the AI arms' first-tick
    // latches, the boomerang whoosh, the missile hum, the bomb blast): the
    // style play's case-2 draws ride the SAME `UpdateProjectiles` stream
    // (`Main.cs:17775-17777`), only after each play's attenuation gate
    // passed. The C# plays inline mid-walk; the port drains at the pass
    // tail (the wave-C batch convention, ANALYSIS "The projectile sound
    // wave").
    for (id, style, x, y, pitch_offset, volume_scale) in app.projectiles.drain_proj_sounds() {
        // S1-P-i08 + W3-i39 — the aiStyle-3 wall thud (`Projectile.cs:19571`)
        // and `AI_007`'s latch dig cue (`Projectile.cs:52757`:
        // `PlaySound(0, i*16, j*16)`) are the int-id Dig play: case 0's
        // `Next(3)` variation + `Next(-10, 11)*0.01` pitch ride the same
        // stream, attenuation gate first (the `play_place_sound` façade's
        // id-0 arm, `LegacySoundPlayer.cs:440-452`, S1-E-21 wave B); the
        // id-2 styles keep the wave-A façade. The d76 DOC-SWEEP volume term
        // rides the id-2/3/4 façades (the Corrupt Whip ×0.6 / Lights Bane
        // `0.15*ai[0]`); the id-0 Dig sites all queue volume 1.
        if id == 0 {
            app.audio.play_place_sound((0, 0), x, y, rng);
        } else if id == 13 {
            // S1-E-21h d75 part 2 + d77 — the Shatter plays: case 13 draws
            // NOTHING (`LegacySoundPlayer.cs:783-792`), attenuation gate
            // first. The queue's sixth term carries the call-site
            // volumeScale: the Crystal Shard's lead (`Projectile.cs:25943`,
            // `PlaySound(13, x, y, 0, 0.35f)`) queues 0.35; the Kill-tail
            // water bottles (`:77018`/`:77033`/`:77048`) and the molotov
            // (`:73923`) queue the stock 1.0 (d77's wall-break int arm
            // shape — both draw-free, the volume rides the queue).
            app.audio.play_shatter_sound(x, y, volume_scale);
        } else if id == 6 {
            // S1-E-21h d77 — the spore cloud's Grass rustle
            // (`Projectile.cs:74517`, `PlaySound(6, pos)`): the wall-break
            // façade's int arm (`LegacySoundPlayer.cs:706-716`, one
            // `Next(-30, 31)` pitch), attenuation gate first.
            app.audio.play_wall_break_sound((id, 1), x, y, rng);
        } else if id == 15 {
            // The yoyo-shot's rare roar (`:36821`, `PlaySound(15, …, 0,
            // 0.35f, jitter)`): case 15 draws nothing (the slot guard is the
            // whole decision); the arm already drew the Next(100)/Next(6)/
            // NextFloat jitter inline — the jitter VALUE and the 0.35
            // volumeScale are dropped here (the recorded deviation).
            app.audio.play_roar(style, x, y);
        } else if id == 36 {
            // The dash-roar twin (`:36825`, `PlaySound(36, …, -1, 0.35f,
            // jitter)`): case 36 draws nothing, style −1 → slot 0 at
            // Pitch += 0.6.
            app.audio.play_dash_roar(style, x, y);
        } else if id == 29 {
            // S1-E-21h d75 part 3 — the Phantasmal Deathray's first-tick
            // zombie-voice cue (`Projectile.cs:31504`, `PlaySound(29, x, y,
            // 104)`): case 29's Style IS the slot (`LegacySoundPlayer.cs:
            // 1008-1050`), the ×0.55 style-104 volume arm, the playing guard
            // BEFORE the one `Next(-10, 11)` pitch — all ride the E21e
            // `play_fighter_moan` façade on the same UpdateProjectiles
            // stream, attenuation gate first. The int overload draws NO
            // `GetRandomPitch` NextFloat.
            // The C# sites truncate (`PlaySound(29, (int)X, (int)Y, …)`
            // `:31504`/`:72989`) — the drain-side truncation is the
            // faithful gate probe (the d75 part-6 receipt).
            app.audio.play_fighter_moan(29, style, x.trunc(), y.trunc(), rng);
        } else if id == 3 && style == 29 {
            // S1-E-21h d75 part 4 — the 1054 SpiderBoulder's death cue
            // (`Projectile.cs:74865`, `PlaySound(SoundID.NPCHit29, x, y)`).
            // d3's `boulder_kill` parks the (3, 29) tuple; the C# call is
            // the STYLE overload (one audio-only GetRandomPitch), so the
            // drain routes to wave C's style façade — case 3's one
            // `Next(-10, 11)` pitch rides this stream, gate first. Other
            // id-3 styles fall through (the façade's None — a future
            // style's route is its own lane's follow-up).
            app.audio.play_npc_hit_sound("NPC_Hit_29", x, y, rng);
        } else if id == 4 && style == -6 {
            // S1-E-21h d75 part 4 + d77 compose — the 1055 Ghoulder's death
            // cue (`Projectile.cs:74869`, `PlaySound(SoundID.NPCDeath6, x,
            // y)`): the STYLE overload — case 4's one `Next(-10, 11)` pitch
            // after the gate (the 10 playing guard is style-10-only), the
            // GetRandomPitch NextFloat audio-only. d3's `boulder_kill`
            // intent rides the −6 named-route sentinel (the −34 precedent,
            // the d75 part-6 receipt): the payload alone cannot tell the
            // style call from d77's kill-tail int `(4, 6)` — the sentinel
            // keeps BOTH overload paths live.
            app.audio.play_npc_death_style(6, x, y, rng);
        } else if id == 4 {
            // S1-E-21h d77 — the kill-tail death cries: the Sharknado bolt
            // (`Projectile.cs:73763`, `PlaySound(4, Center, 19)`), the
            // Phantasmal Sphere (`:73016`, style 6), the Stardust Jellyfish
            // (`:72571`, style 7): the int-overload case-4 façade (one
            // `Next(-10, 11)` pitch after the gate, the 10 playing guard,
            // NO GetRandomPitch).
            app.audio.play_npc_death_int(style, x, y, rng);
        } else if id == 37 {
            // S1-E-21h d75 part 4 — the 502 Meowmere star's bounce chime
            // (`Projectile.cs:18209`, `PlaySound(37, x, y, 5 + ai[0])`):
            // case 37's Next(57,59) slot pick + the Next(-40,41) pitch
            // ride this stream (the int overload — no GetRandomPitch),
            // the Style×0.05 volume scales inside the façade.
            app.audio.play_star_chime(style, x, y, rng);
        } else if id == 2 && style == -34 {
            // S1-E-21h d75 part 4 — the 1106 Foxparks flame's per-life cue
            // (`Projectile.cs:41369`, `PlaySound(SoundID.FoxparksFlame,
            // Center)`): the −34 named-row marker routes to the NAMED
            // (2, 34) row at volume 0.15 (`SoundID.cs:1725`) — the plain
            // +34 tuple stays Item34's (d76's Nebula Blaze latch, volume
            // 1.0). The by-name façade keeps the volume (the
            // play_chillet_sound precedent).
            app.audio.play_foxparks_flame(x, y, rng);
        } else if id == 18 {
            // W4-d1 retire — the falling-block forced placement's coin-pile
            // tail (`WorldGen.cs:60273-60278`: the legacy Coins 18, the int
            // overload — no draws, no pitch, `LegacySoundPlayer.cs:865-869`;
            // the `place_tile_sound` table's 330-333 arm).
            app.audio.play_place_sound((18, 1), x, y, rng);
        } else {
            app.audio.play_proj_sound((id, style), x, y, pitch_offset, volume_scale, rng);
            // S1-E-21h f1 — the queue's sixth term (d77's `volumeScale`
            // seam, adopted from `9d35874a`): the id-0/13/4 int-overload
            // routes ride INSIDE `play_proj_sound_scaled`'s int match (f1's
            // part-4 shape) and ignore the scale (the int overload's own
            // volume is 1 at every f1 site); the id-2 styles fold it into
            // `play_use_sound_scaled`'s post-gate multiply
            // (`LegacySoundPlayer.cs:1411`).
            app.audio
                .play_proj_sound_scaled((id, style), x, y, pitch_offset, volume_scale, rng);
        }
    }
    // W4-e7 lane 23 — the Snapping Stone strike family's queued cues
    // (`TryMakingSnappingStoneStrike`'s particle-body + owner-side plays,
    // `Projectile.cs:14355`/`:14363`/`:14383-14394`): the PVE walk queued
    // them mid-hit (the pool cannot borrow the audio); they drain here on
    // the SAME `UpdateProjectiles` stream (`Main.cs:17775`), the S1-E-21c
    // queue→drain convention. The case-2 draws defer to this tail (the
    // recorded ANALYSIS deviation); the particle dust draws ran inline.
    for sound in app.projectiles.drain_striking_sounds() {
        crate::snapping_stone::play_striking_sound(&mut app.audio, sound, rng);
    }
    // `NewProjectile`'s `TryUpdateChannel` (`Projectile.cs:11097-11100`):
    // the local spawns of this tick track the cancel key's expected type —
    // the last one wins (`ChannelCancelKey.TryTracking` `Player.cs:134-140`).
    // The pool queued the intents (it cannot borrow the player mid-loop);
    // apply them here, after the walk, before the next tick's kills read the
    // key (S1-P-i23).
    if let Some(p) = app.player.as_mut() {
        app.projectiles.apply_channel_tracks(p);
    }
    // `AI_103`'s mark spawn (`Projectile.cs:33467-33475`): the stardust
    // tower's mark resolves into the NPC vec here (the pool can't grow it).
    for (kind, x, y, vx, vy) in app.projectiles.drain_mark_spawns() {
        let k = crate::npc::NpcKind::from_u16(kind).unwrap_or(crate::npc::NpcKind::Slime);
        let mut n = crate::npc::Npc::new(k, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n.vx = vx;
        n.vy = vy;
        n.worm_id = 1;
        app.npcs.push(n);
    }
    // W4-d9 wiring — the aiStyle-89 ritual's 300-tick dragon/squid spawn
    // (`Projectile.cs:32346-32356`): the VANILLA ids 454/521 resolve through
    // `kind_from_net_id` (the bestiary NET-id space, the
    // `spawn_shimmer_release` precedent — NOT `from_u16`'s CS ids). The
    // dragon head (454, the d40 rows) and the squidhead (521, ported from
    // `NPC.cs:15768-15779`) now have port kinds. `NPC.NewNPC`'s Bottom =
    // (X, Y) (`NPC.cs:80770`) — the spawn site subtracts the half-width +
    // the full height (the shark-spawn precedent). The C# stores the
    // returned NPC index into the marker's `ai[1]` (`:32350`/`:32354`);
    // the port's compacted NPC vec re-derives indices each tick, so the
    // write-back rides the current push index (the saucer host-link
    // convention, `world_sim/npcs.rs`'s carrier pre-pass).
    for (slot, npc_type, x, y) in app.projectiles.drain_ritual_npc_spawns() {
        let Some((kind, slime)) = crate::npc::kind_from_net_id(npc_type as i32) else {
            continue;
        };
        let (w, h) = kind.size();
        let mut n = crate::npc::Npc::new(kind, x - w * 0.5, y - h, 1, 0.0);
        n.slime = slime;
        n.hp = n.max_hp;
        n.net_update = true;
        let idx = app.npcs.len();
        app.npcs.push(n);
        // The C#'s `this.ai[1] = NPC.NewNPC(...)` — the host-validity arm
        // (`Projectile.cs:32362-32381`) reads the index next tick.
        app.projectiles.set_ai1(slot, idx as f32);
    }
    // `AI_108`'s 578 bolt spawn (`Projectile.cs:33838-33841`): the lightning
    // marker resolves into the ritual bolt (580) with the marker's payload.
    for (owner, damage, kb, x, y, vx, vy, ai0) in app.projectiles.drain_ritual_bolts() {
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::VORTEX_RITUAL_BOLT,
            owner,
            x,
            y,
            vx,
            vy,
            damage,
            kb,
            ai0,
            0.0,
        );
    }
    // S1-D-3 W-9 fishron wiring — the aiStyle-64 cloud's next segment
    // (`Projectile.cs:29496`): the same kind, ai0 = 10, ai1 - 1, at the
    // segment point (the C#'s NewProjectile centre convention re-centred
    // to the port's top-left spawn).
    for (owner, kind, damage, kb, x, y, vx, vy, ai1) in app.projectiles.drain_cloud_segments() {
        if let Some(row) = crate::projectile::proj_data(kind) {
            app.projectiles.spawn_with_data_ai_owned(
                kind,
                owner,
                x - row.w * 0.5,
                y - row.h * 0.5,
                // The segment inherits the parent's velocity
                // (`:29496`'s `base.velocity.X/Y`).
                vx,
                vy,
                damage,
                kb,
                10.0,
                ai1,
            );
        }
    }
    // The aiStyle-64 cloud's shark spawn (`Projectile.cs:29509-29516`):
    // `NPC.NewNPC`'s Bottom = (X, Y) (`NPC.cs:80771`), the velocity
    // inherited, the 386's ai[2]/ai[3] presets AT THE SPAWN SITE — the
    // style71 machine reads them on its first tick (the sway amplitude
    // and the rise speed).
    for (npc_type, x, y, vx, vy, ai2, ai3) in app.projectiles.drain_shark_spawns() {
        let kind = match npc_type {
            372 => crate::npc::NpcKind::Sharkron,
            373 => crate::npc::NpcKind::Sharkron2,
            _ => continue,
        };
        let (w, h) = kind.size();
        let mut n = crate::npc::Npc::new(kind, x - w * 0.5, y - h, 1, 0.0);
        n.hp = n.max_hp;
        n.vx = vx;
        n.vy = vy;
        n.ai[2] = ai2;
        n.ai[3] = ai3;
        n.net_update = true;
        app.npcs.push(n);
    }
    // W4-d1 — the aiStyle-6 fertilizer cloud's sapling growth
    // (`Projectile.cs:24213-24220`'s `AttemptToGrowTreeFromSapling` calls):
    // the growth draws `WorldGen.genRand`, so the pool arm parked the
    // requests and the pass resolves them on the `UpdateWorldGen` stream
    // (the `palm_frame_rolls` deferral precedent, `world_sim/editing.rs`).
    // The `treeHeightAddon` / `ignoreWalls` args park in the growth port
    // today (`growth/trees.rs`, M5-SEED-1) — the request carries them.
    for (x, y, underground, _addon) in app.projectiles.drain_sapling_grows() {
        let _ = tr_core::growth::attempt_grow_tree_from_sapling(
            &mut app.world,
            x,
            y,
            underground,
            app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
        );
    }
    // S1-P-i35 — the bobber `DoASplash` drain (`Projectile.cs:53893-53905`):
    // case 19's playing guard refuses BEFORE the one pitch draw, on the
    // `UpdateProjectiles` stream (the C# plays inline mid-`AI_061`; the port
    // drains at the pass tail, the S1-E-21c convention).
    for (style, x, y) in app.projectiles.drain_splashes() {
        app.audio.play_splash_sound(style, x, y, rng);
    }
    // W4-d4 — the beach ball's `Game.BallBounceResult` broadcasts
    // (`Projectile.cs:18605-18608`, the type-155 movement arm): the C#
    // `ChatHelper.BroadcastChatMessage(..., ChatColors.ServerMessage)` runs
    // inline mid-`HandleMovement`; the port drains the pool's queued lines
    // at the pass tail (the golf cup-chat precedent).
    for text in app.projectiles.drain_ball_chats() {
        app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
    }
    // W4-d4 — the Mechanical Piranha's `SetDummyItemTime(5)`
    // (`Projectile.cs:27528`): the C# calls it per active piranha per
    // sub-step (idempotent); the port applies ONE dummy-item-time(5) when
    // the pool flagged it. `SetDummyItemTime` (`Player.cs:4432-4441`):
    // itemAnimation = itemTime = frames, itemTimeMax = frames + 1 — the
    // port's swing is the animation, item_time the action cadence (the
    // golf `PlayerWrites` dummy arm, `world_sim/projectiles.rs`).
    if app.projectiles.drain_piranha_dummy() {
        app.swing = 5.0 / 60.0;
        app.swing_max = app.swing;
        app.item_time = 5.0 / 60.0;
    }
    // W4-d10 — the Coin Portal's gold-coin fountain (`Projectile.cs:32932-
    // 32933`): `Item.NewItem` of the Gold Coin 73 (stack 1, prefix 0) at
    // the truncated centre with the arm's EXPLICIT velocity — the
    // RotatedByRandom pair + the NextFloat speed already drew on the
    // `UpdateProjectiles` stream inside the AI; the explicit-velocity arm
    // (`Item.cs:49738-49741`) skips the scatter pair and prefix 0 rolls
    // nothing, so this drain is stream-neutral. The 14×14 item box centres
    // on the spawn point (`WorldItem.Center = center`, `Item.cs:49736`).
    for (x, y, vx, vy) in app.projectiles.drain_coin_drops() {
        app.items.push(crate::item::ItemDrop::new(
            73,
            1,
            0,
            x - crate::item::SIZE * 0.5,
            y - crate::item::SIZE * 0.5,
            vx,
            vy,
            0.0,
        ));
        if app.items.len() > 300 {
            app.items.remove(0);
        }
    }
    // S1-P-i35 — the bobber death hook's catch drain (`Projectile.cs:71986-
    // 71994` → `GiveItemToPlayer` → `QuickSpawnItem` → `GetOrDropItem`): the
    // item fills the owner's inventory first (`Player.GetItem`'s slot walk),
    // the overflow drops at the player centre. The stack roll already ran on
    // the `UpdateProjectiles` stream (the kill hook); the drop's velocity/
    // prefix draws ride the `UpdateItems` stream (`Item.NewItem` under
    // `Main.cs:17779`). The prefix rides the port's pinned velocity-triplet-
    // then-prefix order (`spawn_item_drop_px`), drawn ONLY for the overflow
    // (the C# `GetItem` keeps the rolled prefix, a recorded order deviation).
    for (owner, item, stack) in app.projectiles.drain_catches() {
        if owner != app.local_id as u8 || stack <= 0 {
            continue;
        }
        let max_stack = crate::registry::max_stack(item);
        let mut pending = stack as u32;
        while pending > 0 {
            let Some(slot) = crate::item::slot_for_pickup(&mut app.inventory, item, 0) else {
                break;
            };
            let room = max_stack.saturating_sub(app.inventory[slot].count);
            if room == 0 {
                break;
            }
            let take = pending.min(room);
            app.inventory[slot].count += take;
            pending -= take;
        }
        if pending > 0 {
            if let Some(p) = app.player.as_ref() {
                let (cx, cy) = p.center();
                crate::world_sim::items::spawn_item_drop_px(
                    &mut app.items,
                    app.rngs.stream(crate::app::RngKey::UpdateItems),
                    item,
                    pending,
                    glam::Vec2::new(cx, cy),
                );
            }
        }
    }
    // W4-d1 — the falling-block death placement's item drops
    // (`Kill_ExplodeTiles`'s aiStyle-10 arm, `Projectile.cs:77562-77569`'s
    // `Item.RequestNewItem`): the refused placement / occupied cell drops
    // the block item at the parked pixel point. The C# queues the request
    // under the `UpdateProjectiles` swap and `Main.UpdateItems` resolves
    // it (`Item.NewItem`'s velocity/prefix draws ride the `UpdateItems`
    // stream — the `drain_catches` convention).
    for (item, x, y) in app.projectiles.drain_block_drops() {
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdateItems),
            item,
            1,
            glam::Vec2::new(x, y),
        );
    // W4-d4 — `Kill()`'s world-drop drain (`Projectile.cs:77572-77615` the
    // rope-coil refund ladder, `:77620-77623` the beach ball's return): the
    // pool's death hook queued `(item, stack, base.Center, zero_velocity)`;
    // each spawns an `Item.NewItem` WORLD drop at the recorded centre (NOT
    // a bobber catch — no inventory hand-off). The COIL calls pass no
    // velocity, so the C# draws the pair inline under `Kill()`
    // (`Item.cs:49749-49750`: X = Next(-30,31)*0.1, Y = Next(-40,-15)*0.1 —
    // none of the kinds is `ItemNoGravity`, `ItemID.cs:1320`); the BALL
    // call passes `Vector2.Zero` (`:77622`) — the `velocity.HasValue` arm
    // (`Item.cs:49738-49741`) sets (0,0) and draws NOTHING (the SEAL-VERIFY
    // fix). `prefix: 0` draws nothing. The port defers the spawn to the
    // pass tail on the `UpdateItems` stream (`Main.cs:17779` — the bobber
    // overflow's recorded stream deviation). The `ReserveForLocalPlayer`
    // ownership sets only `enemyGrabDelayTime` in netMode 0
    // (`WorldItem.cs:295-305`), unported — the drop is player-grabbable
    // immediately (the lucky-coin precedent).
    for (item, stack, cx, cy, zero_velocity) in app.projectiles.drain_death_drops() {
        if stack <= 0 {
            continue;
        }
        let (vx, vy) = if zero_velocity {
            (0.0, 0.0)
        } else {
            let rng = app.rngs.stream(crate::app::RngKey::UpdateItems);
            (
                rng.next_range(-30, 31) as f32 * 0.1,
                rng.next_range(-40, -15) as f32 * 0.1,
            )
        };
        // `WorldItem.Center = center` (`Item.cs:49736`) — the port's 14×14
        // drop box centred on the dead projectile's centre (the universal
        // `ItemDrop` box deviation, the bug-net catch precedent).
        let half = crate::item::SIZE * 0.5;
        let drop = crate::item::ItemDrop::new(
            item,
            1,
            glam::Vec2::new(x, y),
        );
    }
    // W4-d6 — the aiStyle-60 slime walk's campfire-validate mismatch kills
    // (`Wiring.ToggleCampFire` → `ValidateTileSquareIsActiveAndOfType`,
    // `Wiring.cs:3012` → `WorldGen.cs:37891-37914`): the object drops spawn
    // on the `UpdateProjectiles` stream (the walk ran inside the pool's
    // update — the actuation-drops precedent, `world_sim/wiring.rs`).
    for drop in app.projectiles.drain_light_drops() {
        if let Some(id) = drop
            .item
            .map(|i| i as u16)
            .or_else(|| crate::registry::tile_drop(drop.tile_kind, 0))
        {
            crate::world_sim::items::spawn_item_drop(&mut app.items, rng, id, 1, drop.x, drop.y);
        }
    }
    // W4-d6 residual — the Spectre-heal orb spawns (`Projectile.ghostHeal`,
    // `Projectile.cs:12095`): the damage walk queued `(owner, x, y, ai0,
    // ai1)` per qualifying hit; the pass spawns each 298 CENTRED on the
    // victim point (`NewProjectile`'s half-extent subtract,
    // `Projectile.cs:10876-10877` — the c6 BladeShot precedent), zero
    // velocity, damage 0, knockback 0. Draw-free (the 298 NewProjectile
    // path rolls nothing — no spawn-roll case, no crit at spawn).
    for (owner, x, y, ai0, ai1) in app.projectiles.drain_heal_orbs() {
        let d = crate::projectile::proj_data(crate::projectile::SPIRIT_HEAL);
        let (hw, hh) = d.map_or((0.0, 0.0), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::SPIRIT_HEAL,
            owner,
            x - hw,
            y - hh,
            0.0,
            0.0,
            0,
            0.0,
            ai0,
            ai1,
        );
    }
    // W4-d6 residual 5 — the Spectre-Mask damage-share spawns
    // (`Projectile.ghostHurt`, `Projectile.cs:12060`): the damage walk
    // queued `(owner, x, y, vx, vy, damage, ai0 target)` per qualifying hit
    // (the direction rolls ran at the hit); the pass spawns each 356
    // CENTRED on the victim point (the `NewProjectile` half-extent
    // subtract), zero knockback, ai0 = the RAW npc index the style-59
    // machine reads. Draw-free spawn (no 356 case in the switch).
    for (owner, x, y, vx, vy, damage, ai0) in app.projectiles.drain_wraths() {
        let d = crate::projectile::proj_data(crate::projectile::SPECTRE_WRATH);
        let (hw, hh) = d.map_or((0.0, 0.0), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::SPECTRE_WRATH,
            owner,
            x - hw,
            y - hh,
            vx,
            vy,
            damage,
            0.0,
            ai0,
            0.0,
        );
    }
    // W4-d6 residual 5 — the Vampire Knives' heal-orb spawns
    // (`Projectile.vampireHeal`, `Projectile.cs:12104`): `(owner, x, y,
    // ai0 owner, ai1 float share)` per qualifying 304 hit; the pass spawns
    // each 305 centred on the victim, zero velocity, damage 0. Draw-free.
    for (owner, x, y, ai0, ai1) in app.projectiles.drain_vampire_orbs() {
        let d = crate::projectile::proj_data(crate::projectile::VAMPIRE_HEAL);
        let (hw, hh) = d.map_or((0.0, 0.0), |d| (d.w * 0.5, d.h * 0.5));
        app.projectiles.spawn_with_data_ai_owned(
            crate::projectile::VAMPIRE_HEAL,
            owner,
            x - hw,
            y - hh,
            0.0,
            0.0,
            0,
            0.0,
            ai0,
            ai1,
        );
    }
    // W4-d6 residual 8 — the gun streams' Kill-splash dust
    // (`Projectile.cs:74029-74147`): the death hook snapshotted the dying
    // 358/406/1042 corpses (the velocity base already written); the drain
    // emits the 50/40/50 dusts on the LIVE DustPool, the explicit gates +
    // the `new_dust` internals on the `UpdateProjectiles` stream in C#
    // order (the g65 gallop-dust pass-tail convention).
    let gun_splashes = app.projectiles.drain_gun_splashes();
    if !gun_splashes.is_empty() {
        let ctx = app.spawn_ctx();
        crate::projectile_gun_splash::drain(
            gun_splashes,
            &mut app.dust,
            app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            &ctx,
            &app.world,
        );
    }
    // S1-P-i55 — the golf arms' queued effects, drained at the pass tail
    // (the `drain_mark_spawns` shape — the pool cannot borrow the App).
    // The C# plays the sounds / trips the wiring mid-walk; the port's
    // batch convention defers them here, on the SAME `UpdateProjectiles`
    // stream (the S1-E-21c precedent, ANALYSIS). The tee placement runs
    // first among the golf effects — the C#'s `TryPlacingAGolfBallNearA
    // NearbyTee` fires inside the helper's AI, ahead of the strike's
    // sounds; the effect queue keeps the arm order.
    for effect in app.projectiles.drain_golf_effects() {
        match effect {
            crate::projectile_ai_golf::GolfEffect::TeePlace { place_position } => {
                crate::projectile_ai_golf::try_place_golf_ball_near_tee_app(app, place_position);
            }
            crate::projectile_ai_golf::GolfEffect::CupScored { hit_location, chat, cue } => {
                // `Wiring.HitSwitch` + `SendData(59)` (`GolfHelper.cs:105-
                // 106`) — the cup's tile trip.
                app.wiring.hit_switch(&mut app.world, hit_location.0, hit_location.1);
                app.send_hit_switch(hit_location.0, hit_location.1);
                // The trip's Mech-28 clicks ride the same stream (the
                // C# call sits inside the `UpdateProjectiles` swap).
                for (px, py) in app.wiring.take_switch_clicks() {
                    app.audio.play_switch_click(
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
                if let Some(text) = chat {
                    app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
                }
                if let Some((id, style, x, y, pitch, vol)) = cue {
                    app.audio.play_proj_sound(
                        (id, style),
                        x,
                        y,
                        pitch,
                        vol,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
            }
            crate::projectile_ai_golf::GolfEffect::PadProbe { pos, old_pos, w, h } => {
                // `Collision.SwitchTiles(..., 4)` (`Projectile.cs:20367`)
                // — the projectile pressure-pad call. The switch sends
                // ride the outcome (`Collision.cs:3304` et al.).
                let outcome = crate::collision::switch_tiles(
                    &mut app.wiring,
                    &mut app.world,
                    pos.x,
                    pos.y,
                    old_pos.x,
                    old_pos.y,
                    w as i32,
                    h as i32,
                    4,
                );
                app.send_switch_outcome(&outcome);
                // The tripped switches' Mech-28 clicks (S1-E-21i): the
                // C#'s `Collision.SwitchTiles` runs inside
                // `UpdateProjectiles` (`Main.cs:17775`'s swap), so the
                // pitch draws ride this pass's stream.
                for (px, py) in app.wiring.take_switch_clicks() {
                    app.audio.play_switch_click(
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                    );
                }
            }
            crate::projectile_ai_golf::GolfEffect::PlayerWrites {
                facing,
                held_proj,
                dummy_item_time,
                cancel_use,
            } => {
                if let Some(p) = app.player.as_mut() {
                    // `player.ChangeDir(direction)` (`Player.cs:42091`) —
                    // the pulley arm is unported (no pulley state); the
                    // facing write is the rest.
                    if let Some(dir) = facing {
                        p.facing = dir;
                    }
                    // `player.heldProj = whoAmI` (`:52490`/`:52535`).
                    if let Some(slot) = held_proj {
                        p.held_proj = slot;
                    }
                    if cancel_use {
                        // `SetItemTime(0)` + `itemAnimation/Max = 0` +
                        // `reuseDelay = 0` (`Projectile.cs:52515-52518`)
                        // — the port's seconds timers.
                        app.item_time = 0.0;
                        app.swing = 0.0;
                        app.swing_max = 0.0;
                        app.attack_cd = 0.0;
                        app.reuse_delay = 0.0;
                        // The channel clears with the helper's death
                        // (`Kill()`'s TryCancelChannel, `Player.cs:52975`)
                        // — the editing pass's release arm owns the
                        // steady-state clear; the cancel is the same tick.
                        p.channel = false;
                        p.channel_item = 0;
                        p.channel_target = Vec2::ZERO;
                    } else if let Some(frames) = dummy_item_time {
                        // `SetDummyItemTime(frames)` (`Player.cs:4432-
                        // 4441`): itemAnimation = itemTime = frames,
                        // itemTimeMax = frames + 1. The port's swing is
                        // the animation, item_time the action cadence.
                        app.swing = frames as f32 / 60.0;
                        app.swing_max = app.swing;
                        app.item_time = frames as f32 / 60.0;
                    }
                }
            }
        }
    }
    // W4-d3 — the aiStyle-25 boulder family's parked requests, drained at the
    // pass tail (the `drain_mark_spawns` shape — the pool cannot borrow the
    // App). The C# rolls the drops/bees and trips the switches inline under
    // the `UpdateProjectiles` swap; the port defers the App-borrowing resolves
    // here, keeping the rolls on this pass's stream (the stack/count/velocity
    // rolls already ran in the kill hook / arm).
    // The item drops (`Item.NewItem`, `Projectile.cs:74952`/`:74956`/`:75096`):
    // the velocity/prefix draws ride the `UpdateItems` stream (`Main.cs:17779`
    // — the bobber-catch precedent); the spawn point is the dying boulder's
    // centre (`NewItem`'s `center` argument).
    for (item, stack, x, y) in app.projectiles.drain_boulder_drops() {
        if stack <= 0 {
            continue;
        }
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            app.rngs.stream(crate::app::RngKey::UpdateItems),
            item,
            stack as u32,
            Vec2::new(x, y),
        );
    }
    // The 655 bee-hive death's bee spawns (`:75116-75118`): `NPC.NewNPC`'s
    // Bottom placement (`NPC.cs:80769-80770` — the Queen-Bee volley precedent,
    // `world_sim/npcs.rs:5507`), the rolled velocity kicks, `netUpdate`.
    // `CanBeReplacedByOtherNPCs` has no port field — dropped (the volley's
    // recorded fold).
    let mut boulder_bees = Vec::new();
    for (kind, x, y, vx, vy) in app.projectiles.drain_boulder_bees() {
        let npc_kind = match kind {
            210 => crate::npc::NpcKind::Bee,
            211 => crate::npc::NpcKind::BeeSmall,
            _ => unreachable!("the hive burst draws only 210/211"),
        };
        let (bw, bh) = npc_kind.size();
        let mut bee = crate::npc::Npc::new(npc_kind, x - bw * 0.5, y - bh, 1, 0.0);
        bee.hp = bee.max_hp;
        bee.vx = vx;
        bee.vy = vy;
        bee.net_update = true;
        boulder_bees.push(bee);
    }
    app.npcs.append(&mut boulder_bees);
    // The `Collision.SwitchTiles` requests the arm parked (the `:17089`
    // objType-3 plate scan, the `:19863`/`:17790` objType-4 pad calls). The
    // scan's plate drops and Mech-28 clicks ride THIS pass's stream (the C#
    // call sits inside `UpdateProjectiles`, `Main.cs:17775`); the `59` sends
    // ride the outcome (the golf PadProbe precedent).
    let boulder_switches: Vec<_> = app
        .projectiles
        .slots_mut()
        .iter_mut()
        .flat_map(|p| std::mem::take(&mut p.wants_boulder_switches))
        .collect();
    for (px, py, ox, oy, w, h, obj_type) in boulder_switches {
        let outcome = crate::collision::switch_tiles(
            &mut app.wiring,
            &mut app.world,
            px,
            py,
            ox,
            oy,
            w,
            h,
            obj_type,
        );
        if let Some((item, tx, ty)) = outcome.dropped {
            crate::world_sim::items::spawn_item_drop_fields(
                &mut app.items,
                app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
                item,
                1,
                tx,
                ty,
            );
        }
        app.send_switch_outcome(&outcome);
        for (cx, cy) in app.wiring.take_switch_clicks() {
            app.audio.play_switch_click(
                cx as f32,
                cy as f32,
                app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            );
        }
    }
}

/// W4-d6 residual 7 — the 298-orb chain end-to-end (written, NOT run —
/// code-only mode): the Spectre-Hood hit queues the intent, the pass-tail
/// drain spawns the orb centred, and the armed style-52 dispatch flies it
/// into the player for the contact heal (`Projectile.cs:28504-28532`).
#[cfg(test)]
mod w4_d6_heal_orb_chain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::{LOST_SOUL, SPIRIT_HEAL};

    #[test]
    fn w4_d6_the_hit_intent_drains_into_a_flying_orb_that_heals_on_contact() {
        // The 8x8 fixture world, the player parked mid-world; a high-life
        // zombie overlapping a Lost Soul shot. The hood flag + the missing
        // health arm the ghostHeal spawn; the drained 298 homes at speed 4
        // through the (v*15+d)/16 blend and heals on the <50f contact.
        let mut app = flow_app(None, vec![], vec![]);
        app.local_id = 0;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 40.0;
        player.y = 40.0;
        player.hp = 50;
        player.max_hp = 100;
        player.ghost_heal = true;
        player.life_steal = 80.0;
        app.player = Some(player);
        let mut npc = crate::npc::Npc::new(crate::npc::NpcKind::Zombie, 56.0, 40.0, 1, 0.0);
        npc.hp = 10_000;
        npc.max_hp = 10_000;
        npc.defense_override = Some(0);
        app.npcs.push(npc);
        // The magic shot overlapping the victim (the 12x12 soul box),
        // flying right at 8 px/tick — it exits the 8x8 world (the
        // out-of-world guard, no Kill) before the id-static 10-tick window
        // expires, so exactly ONE hit lands and ONE orb spawns.
        app.projectiles
            .spawn_with_data_ai_owned(LOST_SOUL, 0, 58.0, 44.0, 8.0, 0.0, 100, 0.0, 0.0, 0.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        // The drain spawned the orb: centred on the POST-strike victim
        // centre (the 6x6 half-extent subtract), ai0 = the local player
        // (the missing-health scan), ai1 = the 0.2 float share.
        let orb = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == SPIRIT_HEAL)
            .expect("the drained 298 lives");
        let n = &app.npcs[0];
        let (nw, nh) = n.size();
        assert_eq!(orb.ai[0], 0.0, "the target scan picked the local player");
        let strike = 10_000 - n.hp;
        assert!(strike > 0, "the hit landed");
        // The drain runs at the pass TAIL, so the orb's first AI tick is
        // the NEXT pass — this tick it holds the spawn velocity 0.
        assert!(orb.vel.length() < 1e-6, "the drain spawns at rest");
        let heal = orb.ai[1];
        assert!((heal - strike as f32 * 0.2).abs() < 1e-3, "the 0.2 share");
        // The next ticks: the armed 52 dispatch blends toward the player
        // (the victim centre is ~15f from the player centre — inside the
        // 50f contact radius, so the box overlap decides; the orb closes
        // the last few px within a tick or two). Fly it home.
        for _ in 0..40 {
            update_projectiles(&mut app, 1.0 / 60.0);
        }
        assert!(
            !app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && p.kind == SPIRIT_HEAL),
            "the contact killed the orb (:28526)"
        );
        let p = app.player.as_ref().unwrap();
        assert!(p.hp > 50, "the contact healed");
        assert!(p.hp <= 100, "the statLifeMax2 cap");
    }
}

/// W4-d4 union — the `Kill()` world-drop drain's wire + stream contract
/// (written, NOT run — code-only): the coil refund ladder
/// (`Projectile.cs:77572-77615`) spawns `Item.NewItem` world drops at the
/// recorded centre, TWO integer draws on the `UpdateItems` stream per drop
/// (`Item.cs:49749-49750`); the beach ball's return (`:77620-77623`) passes
/// `Vector2.Zero` — the `HasValue` arm draws NOTHING (the SEAL-VERIFY
/// fix). Every other stage stream pristine.
#[cfg(test)]
mod w4_d4_death_drop_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::{ROPE_COIL, VINE_ROPE_COIL};
    use tr_core::UnifiedRandom;

    /// A coil hovering inside the 8×8-tile fixture world with the given
    /// placed-count. `time_left = 1` expires it on the first pass tick
    /// (the lifetime block) without any placement contact.
    fn coil_app(kind: u16, placed: f32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let s = app
            .projectiles
            .spawn_with_data_ai(kind, 32.0, 32.0, 0.0, 0.0, 0, 0.0, 0.0, placed)
            .unwrap();
        app.projectiles.slots_mut()[s].time_left = 1;
        app
    }

    /// The unplaced coil returns ONE coil item; the partially placed one
    /// returns `10−ai[1]` rope items — both as WORLD drops at the coil's
    /// centre, not inventory hand-offs (the C# `RequestNewItem`, not the
    /// bobber's `GiveItemToPlayer`).
    #[test]
    fn w4_d4_union_the_coil_expiry_drains_the_refund_as_a_world_drop() {
        let mut app = coil_app(ROPE_COIL, 0.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "one refund drop");
        let d = &app.items[0];
        assert_eq!((d.kind, d.count), (985, 1), "the coil item back (`:77576`)");
        // The 14×14 box centred on (39, 39) — the coil's centre at death.
        assert!((d.center().x - 39.0).abs() < 1e-3 && (d.center().y - 39.0).abs() < 1e-3);
        assert_eq!(d.prefix, 0, "prefix: 0 (`:77576`)");
        assert_eq!(d.grab_delay_time, 0, "netMode 0 sets no player delay");
        // The velocity pair on the UpdateItems stream: X = Next(-30,31)*0.1,
        // Y = Next(-40,-15)*0.1 (`Item.cs:49749-49750`), seed 0.
        let mut expect = UnifiedRandom::new(0);
        let vx = expect.next_range(-30, 31) as f32 * 0.1;
        let vy = expect.next_range(-40, -15) as f32 * 0.1;
        assert!((d.vel.x - vx).abs() < 1e-6 && (d.vel.y - vy).abs() < 1e-6);
        assert_eq!(*app.rngs.stream(RngKey::UpdateItems), expect, "exactly two draws");
        for key in RngKey::ALL {
            if key != RngKey::UpdateItems {
                assert_eq!(
                    *app.rngs.stream(key),
                    UnifiedRandom::new(0),
                    "{} pristine",
                    key.name()
                );
            }
        }
        // The partial refund: ai[1] = 4 → 6 vine-rope items (`:77591`).
        let mut app2 = coil_app(VINE_ROPE_COIL, 4.0);
        update_projectiles(&mut app2, 1.0 / 60.0);
        assert_eq!(app2.items.len(), 1);
        assert_eq!((app2.items[0].kind, app2.items[0].count), (2996, 6));
        // The full budget drops nothing — the drain never runs.
        let mut app3 = coil_app(ROPE_COIL, 10.0);
        update_projectiles(&mut app3, 1.0 / 60.0);
        assert!(app3.items.is_empty(), "10 placed → no refund");
        for key in RngKey::ALL {
            assert_eq!(*app3.rngs.stream(key), UnifiedRandom::new(0), "{} pristine", key.name());
        }
    }

    /// The beach ball's zero-velocity death returns ONE Beach Ball (859)
    /// at `base.Center` (`:77620-77623`) — the same drain, but the call
    /// passes `Vector2.Zero` (`:77622`): the `velocity.HasValue` arm
    /// (`Item.cs:49738-49741`) sets (0,0) and draws NOTHING (the
    /// SEAL-VERIFY fix — the earlier two-draw pin misread the call).
    #[test]
    fn w4_d4_union_the_beach_ball_return_drains_on_the_items_stream() {
        let mut app = flow_app(None, vec![], vec![]);
        app.projectiles
            .spawn_with_data(crate::projectile::BEACH_BALL, 32.0, 32.0, 0.0, 0.0, 0, 0.0);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.items.len(), 1, "the ball returns");
        let d = &app.items[0];
        assert_eq!((d.kind, d.count), (859, 1), "`:77622`");
        // The 44×44 ball's centre: (54, 54.1) — the death hook reads the
        // post-integrate slot (the dry gravity tail's 0.1 dead-tick move,
        // the pool-wide hook timing deviation).
        assert!((d.center().x - 54.0).abs() < 1e-3 && (d.center().y - 54.1).abs() < 1e-3);
        assert!(
            d.vel.x == 0.0 && d.vel.y == 0.0,
            "Vector2.Zero — the HasValue arm sets (0,0)"
        );
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine — the explicit velocity draws nothing",
                key.name()
            );
        }
    }
}

/// S1-E-21h — the pass-tail sound drain's stream contract (written, NOT
/// run — code-only mode): the projectile cue plays consume the
/// `UpdateProjectiles` stream ONLY when the attenuation gate passed, and
/// every other stage stream stays pristine (the E21e witness pattern).
#[cfg(test)]
mod s1_e_21h_sound_stream_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::FROST_BOLT;
    use tr_core::UnifiedRandom;

    fn frost_bolt_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        // Inside the 8x8-tile fixture world, at rest — the arm's first
        // tick queues its `SoundID.Item28` latch cue and nothing else.
        app.projectiles.spawn_with_data_ai_owned(
            FROST_BOLT,
            0,
            32.0,
            32.0,
            0.0,
            0.0,
            10,
            4.0,
            0.0,
            0.0,
        );
        app
    }

    fn listener(sound_volume: f32) -> crate::sound::Listener {
        crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        }
    }

    #[test]
    fn s1_e_21h_the_proj_cue_draws_ride_update_projectiles_only_when_audible() {
        // Muted: the drain plays nothing and the whole pass draws nothing
        // (the frost bolt's arm is RNG-free) — the gate precedes the
        // case-2 switch (`LegacySoundPlayer.cs:369-436` before `:516`).
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine under the muted gate",
                key.name()
            );
        }
        // Audible: exactly ONE case-2 default pitch draw (`Next(-6, 7)`,
        // `:632`) on the `UpdateProjectiles` stream; the GetRandomPitch
        // NextFloat rides the audio-only stream, never a stage stream.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "exactly one case-2 pitch draw on UpdateProjectiles"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
        // The latch holds: the second tick queues nothing, draws nothing.
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "the one-cue-per-life latch draws nothing on tick 2"
        );
    }

    /// S1-E-21h d75 part 2 — the new drain arms' stream contract: the
    /// Crystal Shard's Shatter (id 13) and the yoyo dash's roar pair
    /// (id 15/36) draw NOTHING from the pass stream at the drain (cases
    /// 13/15/36 are draw-free in the legacy player) — the dash's own
    /// `Next(100)` roll already drew inside the arm. The sonar trackable
    /// (id 42) is draw-free too.
    #[test]
    fn s1_e_21h_d75_p2_the_shatter_and_roar_drains_are_draw_free() {
        use crate::projectile::{CRYSTAL_SHARD_MELEE, EYE_OF_CTHULHU_DASH};
        // The lead shard: one Next(100)-free Shatter intent — the whole
        // pass draws nothing (the shard arm is RNG-free).
        let mut app = flow_app(None, vec![], vec![]);
        app.audio.set_listener(listener(1.0));
        app.projectiles.spawn_with_data_ai_owned(
            CRYSTAL_SHARD_MELEE,
            0,
            32.0,
            32.0,
            0.0,
            0.0,
            10,
            4.0,
            0.0,
            0.0,
        );
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine: the Shatter drain draws nothing",
                key.name()
            );
        }
        // The dash: `extraUpdates = 3` — four sub-steps, but the cue
        // block's gate reads `localAI[0] == 0` and the SAME arm call bumps
        // the counter after it (`:36845`), so only the FIRST sub-step
        // rolls Next(100); the drain then adds one case-2 pitch draw for
        // the common Item20 intent (the roar drains draw nothing). The
        // 100-px box would trip the out-of-world guard in the 8x8 fixture
        // world — shrink it (the cue block reads no box state; Center
        // reads pos + w/2).
        let mut app = flow_app(None, vec![], vec![]);
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .spawn_with_data_ai_owned(
                EYE_OF_CTHULHU_DASH,
    /// S1-E-21h f1 — the `Kill()` death cue's stream contract: a shot that
    /// dies this pass queues its death cue, and the case-2 pitch draw rides
    /// the `UpdateProjectiles` stream ONLY when audible. The Jester's Arrow
    /// (5, `:75670` Item10) dies on its first sub-step's `penetrate == 0`
    /// after a hit; here it is spawned with `time_left = 1` so the lifetime
    /// block kills it on tick 1 without a target. Written, NOT run.
    #[test]
    fn s1_e_21h_f1_the_death_cue_draws_ride_update_projectiles_only_when_audible() {
        use crate::projectile::JESTERS_ARROW;
        fn dying_star_app() -> App {
            let mut app = flow_app(None, vec![], vec![]);
            app.projectiles.spawn_with_data_ai_owned(
                JESTERS_ARROW,
                0,
                32.0,
                32.0,
                0.0,
                0.0,
                10,
                4.0,
                0.0,
                0.0,
            )
            .expect("a free slot");
        app.projectiles.slots_mut()[slot].w = 8.0;
        app.projectiles.slots_mut()[slot].h = 8.0;
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        if witness.next_range(0, 100) == 0 {
            let _ = witness.next_range(0, 6);
            let _ = witness.next_float();
            // The roar drain draws nothing.
        } else {
            let _ = witness.next_range(-6, 7);
        }
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "the first sub-step's roll + the intent's drain draw"
            );
            // Force the lifetime kill on tick 1 (the death hook fires).
            let slot = app
                .projectiles
                .slots()
                .iter()
                .position(|p| p.active && p.kind == JESTERS_ARROW)
                .expect("the arrow lives");
            app.projectiles.slots_mut()[slot].time_left = 1;
            app
        }
        // Muted: the death cue queues but the drain's gate refuses before
        // the case-2 switch — no gameplay draw.
        let mut app = dying_star_app();
        app.audio.set_listener(listener(0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine under the muted death-cue gate",
                key.name()
            );
        }
        // Audible: exactly ONE case-2 default pitch draw (`Next(-6, 7)`)
        // on the `UpdateProjectiles` stream — the death cue's Item10.
        let mut app = dying_star_app();
        app.audio.set_listener(listener(1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "exactly one case-2 pitch draw for the death cue"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
    }

    /// S1-E-21h d75 part 4 — the two new drain routes' stream contract:
    /// the RainbowBoulder trackable `(42, 274)` (d3's hard-bounce cue,
    /// `Projectile.cs:20104`) draws NOTHING (case 42,
    /// `LegacySoundPlayer.cs:1376-1381`); the FoxparksFlame named-row
    /// marker `(2, -34)` (`:41369`) draws exactly ONE case-2 default
    /// pitch (`Next(-6, 7)`) — the same draw the plain +34 tuple makes,
    /// only the volume differs (0.15 vs 1.0). Both fire only past the
    /// attenuation gate. The intents are parked through a rowed carrier
    /// (the FROST_BOLT slot at rest — its own first-tick latch pre-fired
    /// via ai[1]; the 1047/1106 rows ride d3/d18's pushed tips).
    #[test]
    fn s1_e_21h_d75_p4_the_rainbow_and_foxparks_drain_routes_draw_correctly() {
        // The trackable: muted refuses draw-free …
        let mut app = frost_bolt_app();
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((42, 274, 32.0, 32.0, 0.0));
        // The death fired once: the second tick draws nothing further.
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "the once-per-death cue draws nothing on tick 2"
        );
    }

    /// S1-E-21h f1 fifth batch — the Kill-tail death cue's stream contract:
    /// the Holy Water bottle (69, `:77018` Shatter) dies this pass and queues
    /// its id-13 cue; the Shatter case draws NOTHING (`LegacySoundPlayer.cs:
    /// 783-792`), so the `UpdateProjectiles` stream stays pristine whether
    /// audible or muted — the gate still precedes the (empty) case. The
    /// bottle is aiStyle 2 (RNG-free flight), spawned `time_left = 1` so the
    /// lifetime block kills it on tick 1. Written, NOT run.
    #[test]
    fn s1_e_21h_f1_b5_the_water_bottle_shatter_cue_is_draw_free() {
        use crate::projectile::HOLY_WATER;
        fn dying_bottle_app() -> App {
            let mut app = flow_app(None, vec![], vec![]);
            app.projectiles.spawn_with_data_ai_owned(
                HOLY_WATER, 0, 32.0, 32.0, 0.0, 0.0, 10, 0.0, 0.0, 0.0,
            );
            let slot = app
                .projectiles
                .slots()
                .iter()
                .position(|p| p.active && p.kind == HOLY_WATER)
                .expect("the bottle lives");
            app.projectiles.slots_mut()[slot].time_left = 1;
            app
        }
        // Audible: the Shatter plays but draws nothing — every stream pristine.
        let mut app = dying_bottle_app();
        app.audio.set_listener(listener(1.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine — the Shatter case draws nothing",
                key.name()
            );
        }
        // Muted: the gate refuses before the (empty) case — still pristine.
        let mut app = dying_bottle_app();
        app.audio.set_listener(listener(0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine under the muted gate",
                key.name()
            );
        }
        // … audible: case 42 draws nothing anywhere.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((42, 274, 32.0, 32.0, 0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine: the trackable drain draws nothing",
                key.name()
            );
        }
        // The FoxparksFlame marker: one case-2 pitch on UpdateProjectiles.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((2, -34, 32.0, 32.0, 0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-6, 7);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "exactly one case-2 pitch draw for the named flame row"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
        // The 1054 SpiderBoulder death cue (3, 29): case 3's ONE
        // `Next(-10, 11)` pitch draw (`LegacySoundPlayer.cs:670`), the
        // GetRandomPitch NextFloat audio-only.
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((3, 29, 32.0, 32.0, 0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "case 3: exactly one pitch draw for the NPCHit29 death cue"
        );
        // The 1055 Ghoulder death cue (4, 6): case 4's ONE
        // `Next(-10, 11)` pitch draw (`:693`), no playing guard (10-only).
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((4, 6, 32.0, 32.0, 0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(-10, 11);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "case 4: exactly one pitch draw for the NPCDeath6 death cue"
        );
        // The 502 Meowmere star chime (37, style): case 37's TWO draws —
        // the Next(57,59) slot pick, then the Next(-40,41) pitch
        // (`LegacySoundPlayer.cs:1328`/`:1332`).
        let mut app = frost_bolt_app();
        app.audio.set_listener(listener(1.0));
        let slot = app
            .projectiles
            .slots()
            .iter()
            .position(|p| p.active)
            .expect("the carrier");
        app.projectiles.slots_mut()[slot].ai[1] = 1.0;
        app.projectiles.slots_mut()[slot].wants_sound = Some((37, 6, 32.0, 32.0, 0.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        let mut witness = UnifiedRandom::new(0);
        let _ = witness.next_range(57, 59);
        let _ = witness.next_range(-40, 41);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            witness,
            "case 37: slot pick then pitch, two draws"
        );
    }
}

#[cfg(test)]
mod w4_c19_ai_light_drain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::TYPHOON;
    use tr_core::UnifiedRandom;

    /// W4-c19 round 3 witness (written, NOT run): the typhoon's rand-gated
    /// light intents drain into the ONE `EntityLights` pair at the pass tail
    /// — the gate draws ride `UpdateProjectiles` ONLY (the `item_lights`
    /// named-stream precedent), every other stage stream stays pristine,
    /// and the intents land in slot + sub-step order with the C# colour.
    #[test]
    fn the_typhoon_intents_drain_into_the_entity_lights_pair() {
        let mut app = flow_app(None, vec![], vec![]);
        // Muted: any spawn-cue play draws nothing (the s1_e_21h precedent),
        // so the stream witness sees ONLY the gate + dust-ring draws.
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        // Past the 10-sub-step gate: the 30x30 box at (32, 32) centres on
        // (47, 47) → tile (2, 2) inside the 8x8 fixture world.
        let slot = app
            .projectiles
            .spawn_with_data_ai_owned(TYPHOON, 0, 32.0, 32.0, 0.0, 0.0, 40, 5.0, 0.0, 0.0)
            .expect("a free slot");
        app.projectiles.slots_mut()[slot].local_ai[1] = 10.0;
        update_projectiles(&mut app, 1.0 / 60.0);
        // The drain emptied the pool seam.
        assert!(app.projectiles.drain_ai_lights().is_empty());
        // The lights: swap the pair, then read the frame's sources. Every
        // intent is the typhoon's (0.1, 0.4, 0.6) at the truncated centre.
        app.entity_lights.end_frame();
        let lights = app.entity_lights.visible();
        assert!(!lights.is_empty(), "at least one gate taken on seed 0");
        for l in lights {
            assert_eq!(l.rgb, [0.1, 0.4, 0.6]);
        }
        // The twin: replay the three sub-steps' gate draws (the taken
        // branches add the 6 × (NextDouble, Next(3,8)) dust-ring pairs).
        let mut twin = UnifiedRandom::new(0);
        let mut taken = 0;
        for _ in 0..3 {
            if twin.next_range(0, 3) == 0 {
                taken += 1;
                for _ in 0..6 {
                    let _ = twin.next_double();
                    let _ = twin.next_range(3, 8);
                }
            }
        }
        assert_eq!(lights.len(), taken, "one light per taken gate");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            twin,
            "the gate + dust-ring draws, nothing else"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert!(
                !app.rngs.is_created(key) || *app.rngs.stream(key) == UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
    }
}

#[cfg(test)]
mod summon_fixture_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::{FROST_HYDRA, HORNET, MAX_PROJECTILES, WOODEN_ARROW};

    /// One sim tick: the player step's summon-budget reset (`Player.cs:28279-28280`
    /// — the live frame runs the player step before the pool pass) then the
    /// pool pass. Driving `update_projectiles` alone would accumulate the
    /// counters across ticks and false-trip the cap.
    fn tick(app: &mut crate::app::App) {
        if let Some(p) = &mut app.player {
            p.num_minions = 0;
            p.slots_minions = 0.0;
        }
        update_projectiles(app, 1.0 / 60.0);
    }

    /// The `TR_SUMMON` fixture arm's once-per-activation shape: the pin
    /// spawns the Hornet + Frost Hydra pair anchored on the player's center,
    /// then resets — further ticks spawn nothing and a re-armed pin spawns
    /// the second pair (the review fix: the arm had no unit test).
    #[test]
    fn the_summon_fixture_spawns_one_pair_per_activation() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let (cx, cy) = app.player.as_ref().unwrap().center();
        app.summon_shot = true;
        tick(&mut app);
        assert!(!app.summon_shot, "the one-shot flag consumed");
        let mut hornet = None;
        let mut hydra = None;
        for p in app.projectiles.slots().iter().filter(|p| p.active) {
            match p.kind {
                HORNET => hornet = Some((p.pos.x, p.pos.y)),
                FROST_HYDRA => hydra = Some((p.pos.x, p.pos.y)),
                _ => {}
            }
        }
        // The same world-bounds clamp the fixture applies (the flow world is
        // 8x8 tiles — an unclamped 80-px Frost Hydra would fall out of world
        // and Phase 3's guard would deactivate it, exactly as the C# would).
        let (ww, wh) = (
            app.world.width as f32 * 16.0,
            app.world.height as f32 * 16.0,
        );
        // The sentry (aiStyle 53) free-falls and `HandleMovement` integrates it
        // on the same tick, so both members stay anchored to within one tick's
        // step rather than exactly.
        let sentry_anchor = (
            (cx + 24.0).clamp(0.0, ww - 80.0 - 1.0),
            (cy - 48.0).clamp(0.0, wh - 74.0 - 1.0),
        );
        let (sx, sy) = hydra.expect("the Frost Hydra renders");
        assert!(
            (sx - sentry_anchor.0).abs() < 8.0 && (sy - sentry_anchor.1).abs() < 8.0,
            "the sentry anchored within one tick's fall: {sx},{sy} vs {sentry_anchor:?}"
        );
        let (hx, hy) = hornet.expect("the Hornet renders");
        let anchor = (
            (cx - 24.0).clamp(0.0, ww - 24.0 - 1.0),
            (cy - 48.0).clamp(0.0, wh - 26.0 - 1.0),
        );
        assert!(
            (hx - anchor.0).abs() < 8.0 && (hy - anchor.1).abs() < 8.0,
            "the Hornet anchored within one tick's drift: {hx},{hy} vs {anchor:?}"
        );
        assert_eq!(app.projectiles.active_count(), 2, "exactly the pair");
        // The flag stays down: further ticks spawn nothing new.
        tick(&mut app);
        assert_eq!(app.projectiles.active_count(), 2, "no per-tick respawn");
        // The reset makes the arm re-armable: a second activation spawns
        // the second pair.
        app.summon_shot = true;
        tick(&mut app);
        assert!(!app.summon_shot);
        assert_eq!(app.projectiles.active_count(), 4, "the second pair");
    }

    /// A full pool declines the spawn — the C# `NewProjectile`'s 1000
    /// sentinel (`Projectile.cs:10793`) drops the request — while the arm
    /// still consumes the flag.
    #[test]
    fn the_summon_fixture_respects_a_full_pool() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Fill every slot with an active filler so the arm finds no space.
        for _ in 0..MAX_PROJECTILES {
            app.projectiles.spawn_with_data_ai_owned(
                WOODEN_ARROW,
                0,
                16.0,
                16.0,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
        }
        app.summon_shot = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(!app.summon_shot, "the flag consumed even on the decline");
        assert_eq!(app.projectiles.active_count(), MAX_PROJECTILES);
        assert!(
            !app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && (p.kind == HORNET || p.kind == FROST_HYDRA)),
            "the pair declined: the pool stayed full of fillers"
        );
    }
}

#[cfg(test)]
mod dw4_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::projectile::WOOD_YOYO;
    use tr_core::UnifiedRandom;

    /// Two same-kind yoyos out: the doubling `flag` lifetime jitter
    /// (`Projectile.cs:67746-67749` — one `Next(10, 31)`) is the pass's
    /// whole draw for the tick.
    fn yoyo_pair_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Inside the 8x8-tile fixture world (the out-of-world guard would
        // deactivate edge spawns on the next tick).
        for i in 0..2 {
            app.projectiles.spawn_with_data_ai_owned(
                WOOD_YOYO,
                0,
                32.0 + i as f32 * 16.0,
                32.0,
                2.0,
                0.0,
                10,
                4.0,
                0.0,
                0.0,
            );
        }
        app
    }

    /// DW-4.1 (the projectiles pass): one tick with two same-kind yoyos out
    /// draws exactly the doubling lifetime jitter's one `Next(10, 31)` on the
    /// `UpdateProjectiles` stream (`Main.cs:17775-17777`), and every other
    /// stage stream stays pristine (the pass's isolation).
    #[test]
    fn test_dw_4_1_projectiles_pass_draws_the_update_projectiles_stream() {
        let mut app = yoyo_pair_app();
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine",
                key.name()
            );
        }
        update_projectiles(&mut app, 1.0 / 60.0);

        // The twin: one `Next(10, 31)` from the pristine seed.
        let mut expect = UnifiedRandom::new(0);
        let _ = expect.next_range(10, 31);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateProjectiles),
            expect,
            "exactly one lifetime-jitter draw on UpdateProjectiles"
        );
        for key in RngKey::ALL {
            if key == RngKey::UpdateProjectiles {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
    }

    /// DW-4.4: the projectiles pass's determinism pair — two same-seed runs
    /// of five ticks give byte-equal slot states and stream states.
    #[test]
    fn test_dw_4_4_projectiles_pass_determinism_pair() {
        let run = || {
            let mut app = yoyo_pair_app();
            // Two ticks: the unchanneld yoyos start their return soon after
            // (the release arm), so the pin holds the pair while both live.
            for _ in 0..2 {
                update_projectiles(&mut app, 1.0 / 60.0);
            }
            let slots: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .map(|p| (p.kind, p.pos, p.vel, p.ai, p.local_ai))
                .collect();
            (slots, app.rngs.stream(RngKey::UpdateProjectiles).clone())
        };
        let a = run();
        let b = run();
        assert_eq!(a.1, b.1, "the stream replayed identically");
        assert_eq!(a.0, b.0, "the pool state is byte-equal");
        assert!(!a.0.is_empty(), "at least one yoyo live across the pair");
    }
}

/// S1-P-i23 — the pass-level channel witnesses (written, NOT run — code-only
/// mode): the yoyo/flail snapshots read the LATCHED `Player.channel` (not a
/// per-tick button re-derive), and the pass tail drains the spawn-tracking
/// intents onto the cancel key (`Projectile.cs:11097-11100`).
#[cfg(test)]
mod s1_p_i23_channel_pass_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::WOOD_YOYO;

    /// A mid-channel player on the Wood Yoyo row (item 3278, shoot 541),
    /// parked at centre (64, 64) inside the 8x8-tile fixture world, the
    /// cursor 40 px right of centre. The button is UP — the latch alone
    /// must hold the aim (the pre-i23 per-tick re-derive would release).
    fn yoyo_channel_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 54.0;
        p.y = 43.0;
        p.channel = true;
        p.channel_item = 3278;
        p.channel_proj_expected = WOOD_YOYO;
        app.player = Some(p);
        app.camera = Vec2::new(64.0, 64.0);
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 40.0, sh * 0.5);
        app.mine_down = false;
        app.projectiles
            .spawn_with_data_ai_owned(WOOD_YOYO, 0, 56.0, 56.0, 0.0, 0.0, 9, 2.5, 0.0, 0.0);
        app
    }

    #[test]
    fn s1_p_i23_the_pass_reads_the_latched_channel_not_the_button() {
        // `AI_099_2_Yoyos`'s release arm (`Projectile.cs:67920-67925`)
        // reads `Main.player[owner].channel` — the latch. Button up +
        // latch open keeps the aim; the latch drop releases it.
        let mut app = yoyo_channel_app();
        update_projectiles(&mut app, 1.0 / 60.0);
        let yoyo = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == WOOD_YOYO)
            .expect("the yoyo lives");
        assert_eq!(yoyo.ai[0], 104.0, "the latched channel holds the aim");
        // The latch drops (a kill-cancel / CC / release would do this).
        app.player.as_mut().unwrap().channel = false;
        update_projectiles(&mut app, 1.0 / 60.0);
        let yoyo = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.kind == WOOD_YOYO)
            .expect("the slot");
        assert_eq!(yoyo.ai[0], -1.0, "the dropped latch releases the aim");
    }

    #[test]
    fn s1_p_i23_the_pass_tail_tracks_the_local_spawns() {
        // `NewProjectile`'s tail (`Projectile.cs:11097-11100`): the pool
        // queues the intents mid-loop; the pass tail applies them — the
        // LAST local spawn of the expected type wins (`TryTracking`
        // `Player.cs:134-140`). Two yoyos out: the key lands on slot 1.
        let mut app = yoyo_channel_app();
        app.projectiles
            .spawn_with_data_ai_owned(WOOD_YOYO, 0, 72.0, 56.0, 0.0, 0.0, 9, 2.5, 0.0, 0.0);
        assert_eq!(
            app.player.as_ref().unwrap().channel_index_expected,
            0,
            "the struct default before the pass"
        );
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.player.as_ref().unwrap().channel_index_expected,
            1,
            "the pass tail tracked the last spawn"
        );
    }
}

/// S1-P-i09 — the yoyo snapshot's `controlUseItem` gates. `Player.ItemCheck`
/// clears the channel whenever the use input is cleared
/// (`Player.cs:42804-42807`): the inventory, the fullscreen map
/// (`Player.cs:24798-24807`) and the minimap hover (`MinimapFrame.cs:164`)
/// all drop it, so the aiStyle-99 release arm (`Projectile.cs:67920-67925`)
/// fires behind them. Re-pinned onto the S1-P-i23 latch: the fixture arms
/// `p.channel` directly (a real use-start would), and the snapshot gates
/// the latch on the use-input clears — no per-tick button re-derive.
#[cfg(test)]
mod s1_p_i09_yoyo_snapshot_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::WOOD_YOYO;

    /// One Wood Yoyo out with the channel latch armed (the Wood Yoyo item
    /// 3278 held). The spawn sits inside the 8×8-tile fixture world (the
    /// out-of-world guard); the camera centres on the player and the mouse
    /// sits right of centre so the aim world point lands inside the string
    /// range (`ai[0]` is a world coordinate, the C#'s own edge quirk).
    fn yoyo_channel_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = crate::player::Player::spawn(&app.world);
        let (cx, cy) = p.center();
        p.channel = true;
        p.channel_item = 3278;
        p.channel_proj_expected = WOOD_YOYO;
        app.player = Some(p);
        app.camera = Vec2::new(cx, cy);
        app.zoom = 1.0;
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 100.0, sh * 0.5);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 3278,
            name: "Wood Yoyo",
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        app.projectiles.spawn_with_data_ai_owned(
            WOOD_YOYO, 0, 32.0, 32.0, 2.0, 0.0, 10, 4.0, 0.0, 0.0,
        );
        app
    }

    #[test]
    fn s1_p_i09_the_held_channel_keeps_the_yoyo_flying() {
        let mut app = yoyo_channel_app();
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(app.projectiles.slots()[0].ai[0] >= 0.0, "the hold keeps the flight");
    }

    #[test]
    fn s1_p_i09_the_open_inventory_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        app.inventory_open = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the inventory clears the use input");
        assert_eq!(app.projectiles.slots()[0].ai[1], 0.0);
    }

    #[test]
    fn s1_p_i09_the_fullscreen_map_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        let mut map = crate::map::WorldMap::new(&app.world);
        map.open = true;
        app.map = Some(map);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the fullscreen map clears the use input");
    }

    #[test]
    fn s1_p_i09_the_minimap_hover_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        app.minimap_hover = Some(crate::ui::minimap::MinimapButton::ZoomIn);
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the minimap hover clears the use input");
    }

    /// `Projectile.cs:67920` — the snapshot carries the owner's `CCed`
    /// (`Player.cced()` — frozen / webbed / stoned): a frozen owner
    /// releases the string even with the button held.
    #[test]
    fn s1_p_i09_the_frozen_owner_releases_the_yoyo() {
        let mut app = yoyo_channel_app();
        app.player.as_mut().unwrap().frozen = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[0].ai[0], -1.0, "the CC releases");
    }
}
})

/// W4-d9 wiring — the aiStyle-89 ritual's 300-tick dragon/squid spawn drain
/// (`Projectile.cs:32346-32356`): the VANILLA ids 454/521 resolve through
/// `kind_from_net_id` into real NPC spawns (the d40 dragon rows + the
/// ported 521 squidhead), and the returned NPC index writes back into the
/// marker's `ai[1]` (the C#'s `this.ai[1] = NPC.NewNPC(...)`). WRITTEN, NOT
/// RUN (code-only mode).
#[cfg(test)]
mod w4_d9_ritual_spawn_drain_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::CULTIST_RITUAL;

    /// A ritual marker parked at (40, 40) inside the 8x8-tile (128-px)
    /// fixture world, with a live cultist host at NPC index 0. The
    /// 300-tick spawn request is seeded directly (the AI arm's own
    /// witness pins the 300-tick gate; this pins the DRAIN).
    fn ritual_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut host = crate::npc::Npc::new(crate::npc::NpcKind::AncientCultist, 10.0, 10.0, 1, 0.0);
        host.hp = host.max_hp;
        app.npcs.push(host);
        let slot = app.projectiles
            .spawn_hostile_ai(CULTIST_RITUAL, Vec2::new(40.0, 40.0), Vec2::ZERO, 0, 0.0, 0.0, 0.0)
            .expect("the 490 row spawns");
        app.projectiles.slots_mut()[slot].wants_ritual_npc_spawn = Some((454, 60.0, 80.0));
        app
    }

    #[test]
    fn w4_d9_the_drain_spawns_the_dragon_head_and_writes_ai1() {
        let mut app = ritual_app();
        update_projectiles(&mut app, 1.0 / 60.0);
        // The dragon head spawned at the Bottom convention: centre X 60,
        // bottom Y 80 → top-left (60 - 16, 80 - 32) = (44, 48).
        let dragon = app.npcs.iter().find(|n| n.kind == crate::npc::NpcKind::CultistDragonHead)
            .expect("the 454 resolved through kind_from_net_id");
        assert_eq!((dragon.x, dragon.y), (44.0, 48.0), "NPC.NewNPC's Bottom = (X, Y)");
        assert_eq!(dragon.hp, dragon.max_hp, "the spawn is at full life");
        // The marker's ai[1] carries the returned NPC index (the host-
        // validity arm reads it next tick).
        let marker = app.projectiles.slots().iter().find(|p| p.active && p.kind == CULTIST_RITUAL)
            .expect("the marker lives (its host is valid)");
        let idx = app.npcs.iter().position(|n| n.kind == crate::npc::NpcKind::CultistDragonHead).unwrap();
        assert_eq!(marker.ai[1], idx as f32, "the ai[1] write-back");
        assert!(marker.wants_ritual_npc_spawn.is_none(), "the intent drained");
    }

    #[test]
    fn w4_d9_the_drain_spawns_the_squidhead_when_requested() {
        let mut app = ritual_app();
        let slot = app.projectiles.slots().iter().position(|p| p.active && p.kind == CULTIST_RITUAL).unwrap();
        app.projectiles.slots_mut()[slot].wants_ritual_npc_spawn = Some((521, 60.0, 80.0));
        update_projectiles(&mut app, 1.0 / 60.0);
        // The squidhead's 60x60 box: top-left (60 - 30, 80 - 60) = (30, 20).
        let squid = app.npcs.iter().find(|n| n.kind == crate::npc::NpcKind::AncientCultistSquidhead)
            .expect("the 521 resolved through kind_from_net_id");
        assert_eq!((squid.x, squid.y), (30.0, 20.0));
        assert_eq!(squid.hp, 2000, "the squidhead's lifeMax");

        }
        }/// W4-f32 — the projectile-side Lucky Coin proc's pass-tail drain
/// (`Projectile.cs:13685-13697` → `drain_lucky_coin_drops`). WRITTEN NOT RUN
/// (code-only mode). The six rolls ride the `UpdateProjectiles` stream
/// inline mid-walk; the drain spawns the `ItemDrop` (stack 1, prefix 0, the
/// explicit velocity) and draws NOTHING — the `UpdateItems` stream stays
/// pristine (the explicit velocity skips `Item.NewItem`'s spread draws,
/// `Item.cs:49738-49740`).
mod f32_lucky_coin_drain_tests {
    use crate::app::RngKey;
    fn f32_the_lucky_coin_drain_spawns_the_drop_draw_free() {
        app.player = Some(crate::player::Player::spawn(&app.world));
        // A pre-rolled proc intent (the pool's mid-walk rolls are pinned by
        // `f32_lucky_coin_proj_tests`): silver 72 at (60, 40), vx 4, vy -2.
        app.projectiles.push_lucky_coin_drop_for_test((72, 60.0, 40.0, 4.0, -2.0));
        assert_eq!(app.items.len(), 1, "the drain spawned the coin");
        let coin = &app.items[0];
        assert_eq!(coin.kind, 72);
        assert_eq!(coin.count, 1, "stack 1 — the C# discards its Next(1, 11)");
        assert_eq!(coin.prefix, 0);
        // `WorldItem.Center = center` — the 14x14 box centred on the NPC.
        assert!((coin.pos.x - (60.0 - crate::item::SIZE * 0.5)).abs() < 1e-5);
        assert!((coin.pos.y - (40.0 - crate::item::SIZE * 0.5)).abs() < 1e-5);
        assert!((coin.vel.x - 4.0).abs() < 1e-5);
        assert!((coin.vel.y - -2.0).abs() < 1e-5);
        assert_eq!(coin.phase, 0.0, "the explicit-velocity spawn draws no phase");
        // The spawn drew nothing: the item stream is still the pristine seed.
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateItems),
            tr_core::UnifiedRandom::new(0),
            "the drain is draw-free"
        );
        // The queue emptied (a second tick spawns nothing new).
        assert_eq!(app.items.len(), 1, "the drain consumed the intent");
    }
}
