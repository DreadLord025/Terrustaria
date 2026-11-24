//! W4-d54 — WorldGen editing-lane dust SUPPLEMENT WITNESSES.
//! The bodies (cutter/unlock-door/paint/coat/ash-shake) moved to their
//! canonical tr-core home `tr_core::worldgen_dust` (g14's port,
//! research/66 union exec): d54's `ash_shake` DROPPED the gore
//! sibling's ARGUMENT draws (`RandomVector2(-40,40)` + `61+Next(3)`,
//! `WorldGen.cs:29669`), breaking the !hitTree stream by 3 draws/row —
//! g14 consumes them (the g17 argument/body hand-off) and its census
//! witnesses are the authority. The LIVE cutter seams
//! (`editing/wiring.rs`) call `kill_actuator_dust`/`kill_wire_dust`
//! directly; the latent bodies' seams arrive with their lanes.
//!
//! This module keeps d54's seven oracle witnesses as supplements
//! (research/66 §3), re-pointed at the tr-core bodies — the ash
//! census oracle ADAPTED to replay the gore argument draws g14
//! consumes. WRITTEN NOT RUN (code-only).

#[cfg(test)]
mod tests {
    //! Each mirror replays the C# draw census on a probe
    //! `UnifiedRandom` and compares `peek()` against the ported
    //! body's stream, then checks the post-spawn field writes.

    use tr_core::particles::{Color, DustPool, SpawnCtx};
    use tr_core::worldgen_dust::{
        coating_color, kill_actuator_dust, kill_wire_dust, paint_coat_effect_dust,
        paint_effect_dust, tree_grow_fx_ash_shake, unlock_door_dust,
    };
    use tr_core::{UnifiedRandom, World};
    use glam::Vec2;

    /// A permissive whole-world ctx (the d52 `dust_ctx` house pattern):
    /// the pool never culls, so every spawn lands and the draw census
    /// matches the live empty-pool path.
    fn dust_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    /// One `NewDust` default block on an empty pool under the 50 % band
    /// (`Dust.cs:153-193`): the two position `Next(box-4)` rolls (the box
    /// clamps to >= 5), the velocity `Next(-20,21)` pair, the frame
    /// `Next(3)`, the scale `Next(-20,21)`; then the per-type tweak — the
    /// 6/228 family adds one `Next(-10,6)` (`Dust.cs:214-220`); 50/11/
    /// 143/31 add nothing. ChildSafety is disabled (the pool default).
    fn block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(dust_type, 6 | 228) {
            r.next_range(-10, 6);
        }
    }

    /// `Utils.RandomVector2` — two `NextDouble` draws.
    fn rv2(r: &mut UnifiedRandom) {
        r.next_double();
        r.next_double();
    }

    /// `Utils.NextVector2Circular` — the unit `NextFloat` + the radius
    /// `NextFloat`.
    fn circular(r: &mut UnifiedRandom) {
        r.next_float();
        r.next_float();
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    /// `WorldGen.cs:58032-58035` — the cutter burst is exactly five
    /// type-50 default blocks, no post draws; the actuator and wire
    /// entry points share the body (the wiring.rs seams).
    #[test]
    fn w4_d54_cutter_burst_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..200 {
            if i % 2 == 0 {
                kill_actuator_dust(10, 20, &mut rng, &mut dust, &ctx, &w);
            } else {
                kill_wire_dust(10, 20, &mut rng, &mut dust, &ctx, &w);
            }
            for _ in 0..5 {
                block(&mut probe, 50, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "cutter stream @{i}");
            drain(&mut dust);
        }
        assert_ne!(rng.peek(), UnifiedRandom::new(54).peek(), "the burst drew");
    }

    /// `WorldGen.cs:37615-37626` — the unlock burst is 3 rows × 4
    /// type-11 blocks; the puffs land at the row's tile corner + the
    /// default block's `Next(12)+4` offsets.
    #[test]
    fn w4_d54_unlock_door_burst_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            unlock_door_dust(12, 30, &mut rng, &mut dust, &ctx, &w);
            for _ in 0..12 {
                block(&mut probe, 11, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "unlock stream @{i}");
            drain(&mut dust);
        }
        // The first puff's position: (12*16 + Next(12)+4, 30*16 + Next(12)+4).
        let mut replay = UnifiedRandom::new(54);
        let px = 192.0 + replay.next_max(12) as f32 + 4.0;
        let py = 480.0 + replay.next_max(12) as f32 + 4.0;
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(54);
        unlock_door_dust(12, 30, &mut rng2, &mut dust2, &ctx, &w);
        let d = dust2.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 11);
        assert_eq!((d.position.x, d.position.y), (px, py));
        // Row 2's first puff (index 8) sits two tiles lower: 512 + Next(12)+4.
        let d8 = dust2.slot(8);
        assert!(d8.position.y >= 516.0 && d8.position.y < 528.0);
    }

    /// `WorldGen.cs:44072-44084` — ten type-143 blocks on `Main.rand`,
    /// each followed by ONE `genRand.Next(2)`; the two streams never
    /// interleave. Both post arms fire across the replay.
    #[test]
    fn w4_d54_paint_effect_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut gen_probe = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let mut saw_big = false;
        let mut saw_small = false;
        for i in 0..100 {
            paint_effect_dust(10, 20, 9, 0, &mut rng, &mut gen, &mut dust, &ctx, &w);
            for _ in 0..10 {
                block(&mut probe, 143, 16, 16);
                if gen_probe.next_max(2) == 0 {
                    saw_big = true;
                } else {
                    saw_small = true;
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "paint main stream @{i}");
            assert_eq!(gen.peek(), gen_probe.peek(), "paint gen stream @{i}");
            drain(&mut dust);
        }
        assert!(saw_big && saw_small, "both Next(2) arms exercised");
    }

    /// `WorldGen.cs:44066-44071` + `:43921-44034` — the colour fold
    /// (`color == 0` takes `oldColor`) and the alpha-150 color-30 arm
    /// (the union fix, research/66 §2); the post-arm scale writes land
    /// on the spawned slot.
    #[test]
    fn w4_d54_paint_effect_writes_the_color_and_scale() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        paint_effect_dust(10, 20, 0, 30, &mut rng, &mut gen, &mut dust, &ctx, &w);
        // Replay the first spawn: the block's scale roll is the LAST
        // Next(-20,21); re-derive it.
        let mut replay2 = UnifiedRandom::new(54);
        replay2.next_max(12);
        replay2.next_max(12);
        replay2.next_range(-20, 21);
        replay2.next_range(-20, 21);
        replay2.next_max(3);
        let scale = 1.0 + replay2.next_range(-20, 21) as f32 * 0.01;
        let mut gen_replay = UnifiedRandom::new(7);
        let big = gen_replay.next_max(2) == 0;
        let d = dust.slot(0);
        assert_eq!(d.color, Color::new(200, 200, 200, 150));
        assert_eq!(d.alpha, 50);
        assert_eq!(d.no_gravity, big);
        let want = if big { scale * 1.2 } else { scale * 0.5 };
        assert!((d.scale - want).abs() < 1e-6);
        // The non-zero color keeps its own id (no fold): paint 1 is
        // red at alpha 255 (the color-30 alpha arm is the ONLY one).
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(54);
        let mut gen2 = UnifiedRandom::new(7);
        paint_effect_dust(10, 20, 1, 30, &mut rng2, &mut gen2, &mut dust2, &ctx, &w);
        assert_eq!(dust2.slot(0).color, Color::new(255, 0, 0, 255));
    }

    /// `WorldGen.cs:44090-44107` — coat 0 with a non-empty old-colour
    /// list draws `Next(count)` BEFORE each spawn; coats 1/2 draw
    /// nothing extra and carry the fixed coating colours.
    #[test]
    fn w4_d54_paint_coat_effect_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let olds = [Color::new(1, 2, 3, 255), Color::new(4, 5, 6, 255)];
        for coat in [0u8, 1, 2, 3] {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut gen = UnifiedRandom::new(7);
            let mut gen_probe = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            for i in 0..50 {
                paint_coat_effect_dust(10, 20, coat, &olds, &mut rng, &mut gen, &mut dust, &ctx, &w);
                for _ in 0..10 {
                    if coat == 0 {
                        probe.next_max(2); // Next(oldColors.Count)
                    }
                    block(&mut probe, 143, 16, 16);
                    gen_probe.next_max(2);
                }
                assert_eq!(rng.peek(), probe.peek(), "coat {coat} main @{i}");
                assert_eq!(gen.peek(), gen_probe.peek(), "coat {coat} gen @{i}");
                drain(&mut dust);
            }
        }
        // The fixed coating colours + the transparent default.
        assert_eq!(coating_color(1), Color::new(235, 170, 255, 255));
        assert_eq!(coating_color(2), Color::new(180, 245, 255, 255));
        assert_eq!(coating_color(0), Color::TRANSPARENT);
        assert_eq!(coating_color(9), Color::TRANSPARENT);
    }

    /// `WorldGen.cs:29642-29671` — the ash-shake's full draw census:
    /// per row the two short-circuit `Next(3)` gates, then per puff the
    /// position `RandomVector2` (2), the type-6 scale argument (1, before
    /// the block), the 4×4→5-clamped default block (+ the type-6
    /// `Next(-10,6)` tweak), the velocity `Circular` (2) and the ember's
    /// `Next(2)` gate; then the !hitTree gore sibling's ARGUMENT draws
    /// (`RandomVector2(-40,40)` = 2 NextDouble + `Next(3)`, `:29669` —
    /// ADAPTED per research/66: g14 consumes them, d54's original
    /// oracle lacked them). Both `hitTree` arms replayed from several
    /// seeds.
    #[test]
    fn w4_d54_ash_shake_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        for hit_tree in [false, true] {
            for seed in [54, 1, 99] {
                let height = 20;
                let mut rng = UnifiedRandom::new(seed);
                let mut probe = UnifiedRandom::new(seed);
                let mut dust = DustPool::new();
                tree_grow_fx_ash_shake(100, 100, height, hit_tree, &mut rng, &mut dust, &ctx, &w);
                let mut num = 0i32;
                while num > -height {
                    if num >= -height / 2 && probe.next_max(3) != 0 {
                        for _ in 0..if hit_tree { 10 } else { 5 } {
                            rv2(&mut probe);
                            block(&mut probe, 31, 4, 4);
                            circular(&mut probe);
                        }
                    }
                    if num as f32 >= (-height) as f32 * 0.7 && probe.next_max(3) != 0 {
                        for _ in 0..if hit_tree { 16 } else { 10 } {
                            rv2(&mut probe);
                            probe.next_float(); // the scale argument
                            block(&mut probe, 6, 4, 4);
                            circular(&mut probe);
                            probe.next_max(2);
                        }
                    }
                    if !hit_tree {
                        // `:29669` — the gore argument draws (g17's body).
                        rv2(&mut probe);
                        probe.next_max(3);
                    }
                    num -= 1;
                }
                assert_eq!(
                    rng.peek(),
                    probe.peek(),
                    "ash shake hit={hit_tree} seed={seed}"
                );
                drain(&mut dust);
            }
        }
    }

    /// `WorldGen.cs:29657-29664` — the ember's post writes: velocity is
    /// the Circular + (0,-0.6) overwrite (the spawn's own velocity is
    /// dead), the `Next(2)` pass doubles it and lifts gravity, then the
    /// constant `(0, hitTree ? -1 : -2)` lift always lands. The gore
    /// argument draws run AFTER both loops — the first ember's replay
    /// is unaffected.
    #[test]
    fn w4_d54_ash_shake_writes_the_ember_fields() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        // height 1: row 0 passes BOTH position gates; find a seed whose
        // first gate passes so the first spawn is a type-31 puff and the
        // ember lane starts at index 5 (hitTree=false).
        tree_grow_fx_ash_shake(100, 100, 1, false, &mut rng, &mut dust, &ctx, &w);
        let mut probe = UnifiedRandom::new(54);
        let gate31 = probe.next_max(3) != 0;
        let mut idx = 0usize;
        if gate31 {
            for _ in 0..5 {
                rv2(&mut probe);
                block(&mut probe, 31, 4, 4);
                circular(&mut probe);
                idx += 1;
            }
        }
        let gate6 = probe.next_max(3) != 0;
        if gate6 {
            // First ember: replay its draws to recover the velocity.
            rv2(&mut probe);
            probe.next_float(); // scale arg
            block(&mut probe, 6, 4, 4);
            let angle = std::f32::consts::PI * 2.0 * probe.next_float();
            let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
            let r = probe.next_float();
            let mut vel = Vec2::new(unit.x * 2.0 * r, unit.y * 2.0 * r) + Vec2::new(0.0, -0.6);
            let doubled = probe.next_max(2) == 0;
            if doubled {
                vel *= 2.0;
            }
            vel += Vec2::new(0.0, -2.0);
            let d = dust.slot(idx);
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.no_gravity, doubled);
            assert!((d.velocity.x - vel.x).abs() < 1e-5);
            assert!((d.velocity.y - vel.y).abs() < 1e-5);
        }
    }
}
