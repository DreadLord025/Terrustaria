"""M4-NPC-5 part F: the teleport detection fix (per-tick jump vs fall)."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

p = 'crates/tr-app/src/npc.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        let mut rng = 7u64;
        let mut teleported = false;
        let mut shot = false;
        for _ in 0..400 {
            tim.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if let Some((kind, sx, sy)) = tim.wants_sphere {
                assert_eq!(kind, NpcKind::ChaosBall);
                // The Chaos Ball spawn point: Tim's bottom-centre, 8 px
                // above the top (`:21242`'s NewNPC(x + w/2, y - 8) with the
                // `Bottom` semantics).
                let (tw, _) = tim.size();
                assert!((sx - (tim.x + tw * 0.5)).abs() < 0.01);
                assert!((sy - (tim.y - 8.0)).abs() < 0.01);
                shot = true;
                break;
            }
            if !teleported {
                let moved = (tim.x - start.0).abs() + (tim.y - start.1).abs();
                if moved > 0.0 {
                    teleported = true;
                    // The teleport respected the ±5-tile telefrag gate.
                    let dcx = ((tim.x + tim.size().0 * 0.5) - pc.0).abs();
                    let dcy = ((tim.y + tim.size().1 * 0.5) - pc.1).abs();
                    assert!(
                        dcx > 70.0 || dcy > 70.0,
                        "the telefrag gate held ({dcx}x{dcy})"
                    );
                    // The feet rest on a solid floor.
                    let floor = w
                        .tile(
                            (tim.x / 16.0) as i32,
                            ((tim.y + tim.size().1) / 16.0) as i32,
                        )
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
                    assert!(floor, "the teleport landed the feet on a solid floor");
                }
            }
        }
        assert!(teleported, "the caster never teleported");
        assert!(shot, "Tim never fired the Chaos Ball");'''
assert old in s, 'caster test loop anchor'
s = s.replace(old, '''        let mut rng = 7u64;
        let mut teleported = false;
        let mut shot = false;
        let _ = start;
        for _ in 0..400 {
            let (px, py) = (tim.x, tim.y);
            tim.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if let Some((kind, sx, sy)) = tim.wants_sphere {
                assert_eq!(kind, NpcKind::ChaosBall);
                // The Chaos Ball spawn point: Tim's bottom-centre, 8 px
                // above the top (`:21242`'s NewNPC(x + w/2, y - 8) with the
                // `Bottom` semantics).
                let (tw, _) = tim.size();
                assert!((sx - (tim.x + tw * 0.5)).abs() < 0.01);
                assert!((sy - (tim.y - 8.0)).abs() < 0.01);
                shot = true;
                break;
            }
            if !teleported {
                // The teleport is a per-tick jump (the gravity fall moves
                // at most MAX_FALL = 8 px a tick, the stand decay < 1).
                let jumped = (tim.x - px).abs() + (tim.y - py).abs() > 16.0;
                if jumped {
                    teleported = true;
                    // The teleport respected the ±5-tile telefrag gate.
                    let dcx = ((tim.x + tim.size().0 * 0.5) - pc.0).abs();
                    let dcy = ((tim.y + tim.size().1 * 0.5) - pc.1).abs();
                    assert!(
                        dcx > 70.0 || dcy > 70.0,
                        "the telefrag gate held ({dcx}x{dcy})"
                    );
                    // The feet rest on a solid floor.
                    let floor = w
                        .tile(
                            (tim.x / 16.0) as i32,
                            ((tim.y + tim.size().1) / 16.0) as i32,
                        )
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
                    assert!(floor, "the teleport landed the feet on a solid floor");
                }
            }
        }
        assert!(teleported, "the caster never teleported");
        assert!(shot, "Tim never fired the Chaos Ball");''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('teleport detection fixed')
