import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

def apply(path, steps):
    s = io.open(path, encoding='utf-8').read()
    changed = False
    for tag, old, new in steps:
        if new in s:
            print(f'{path} {tag}: already')
            continue
        assert old in s, f'{path} {tag}: ANCHOR MISSING'
        s = s.replace(old, new, 1)
        changed = True
        print(f'{path} {tag}: ok')
    if changed:
        io.open(path, 'w', encoding='utf-8', newline='').write(s)

# ============ npc.rs: the 5 kinds ============
P = 'crates/tr-app/src/npc.rs'
GOLEMS = 'NpcKind::Golem'
apply(P, [
    ('enum',
     '''    /// `NPCID.GoblinTinkerer` (107) — the freed Tinkerer (`SetDefaults`
    /// `NPC.cs:10271-10283`: 18x40, aiStyle 7 the town walker, 250 HP,
    /// def 15, kb 0.5).
    GoblinTinkerer,
}''',
     '''    /// `NPCID.GoblinTinkerer` (107) — the freed Tinkerer (`SetDefaults`
    /// `NPC.cs:10271-10283`: 18x40, aiStyle 7 the town walker, 250 HP,
    /// def 15, kb 0.5).
    GoblinTinkerer,
    /// `NPCID.Golem` (245) — the BODY (`SetDefaults` `NPC.cs:12163-12177`:
    /// 140x140, aiStyle 45, dmg 72, def 26, 15000 HP, kb 0, boss, value
    /// 15 gold). Its death is the fight's end (`:79346`'s event flag).
    Golem,
    /// `NPCID.GolemHead` (246) — the attached head (`:12178-12192`: 70x70,
    /// aiStyle 46, dmg 64, def 20, 25000 HP, the silent death — the head
    /// detaches rather than dies).
    GolemHead,
    /// `NPCID.GolemFistLeft` (247) — the lunging fist (`:12193-12206`:
    /// 40x30, aiStyle 47, dmg 59, def 28, 10000 HP).
    GolemFistLeft,
    /// `NPCID.GolemFistRight` (248) — the mirrored fist (the same stats).
    GolemFistRight,
    /// `NPCID.GolemHeadFree` (249) — the detached head (`:12207-12220`:
    /// 70x70, aiStyle 48, dmg 80, def 32, 16000 HP, `dontTakeDamage` —
    /// the spawn-shielded free-flying phase).
    GolemHeadFree,
}'''),
    # the dispatch: the golem machine arm
    ('dispatch',
     '''            NpcKind::GoblinArcher => self.update_archer(world, player, rng),''',
     '''            NpcKind::GoblinArcher => self.update_archer(world, player, rng),
            NpcKind::Golem
            | NpcKind::GolemHead
            | NpcKind::GolemFistLeft
            | NpcKind::GolemFistRight
            | NpcKind::GolemHeadFree => self.update_golem(world, player),'''),
    # the machine (before update_brain)
    ('machine',
     '''    /// The Creeper (`AI_055`, `NPC.cs:32657-32735`): the orbit around''',
     '''    /// The Golem family's v1 machines (the aiStyles 45-48): the body
    /// hovers the EoC-cycle shape toward the player; the attached head
    /// floats at the body's crown (the offset lock); the fists lunge
    /// (the reach at the 120-tick interval, the retract); the free head
    /// clears its spawn shield at 120 ticks and hovers. The full AI_045
    /// through AI_048 (the fireballs, the head-retract, the laser eyes)
    /// = the follow-ups.
    fn update_golem(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        match self.kind {
            NpcKind::Golem => {
                // The body: the EoC-cycle shape's hover arm (the target
                // point 200 above the player, the +-0.07 accels, the 8
                // cap — the established v1 hover).
                let tx = pcx - cx;
                let ty = (pcy - 200.0) - cy;
                let d = (tx * tx + ty * ty).sqrt().max(1.0);
                self.vx += (tx / d) * 0.07;
                self.vy += (ty / d) * 0.07;
                self.vx = self.vx.clamp(-8.0, 8.0);
                self.vy = self.vy.clamp(-8.0, 8.0);
            }
            NpcKind::GolemHead => {
                // The attached head: the same hover target as the body —
                // they spawn together and hold formation (the offset
                // lock through the roster link = the follow-up).
                let tx = pcx - cx;
                let ty = (pcy - 200.0) - cy;
                let d = (tx * tx + ty * ty).sqrt().max(1.0);
                self.vx += (tx / d) * 0.07;
                self.vy += (ty / d) * 0.07;
                self.vx = self.vx.clamp(-8.0, 8.0);
                self.vy = self.vy.clamp(-8.0, 8.0);
            }
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => {
                // The fists: the lunge cycle — the reach toward the
                // player at the 120-tick interval (the ai[0] counter),
                // the straight flight, the decay to the stop.
                if self.ai[0] <= 0.0 {
                    self.caster_counter += 1;
                    if self.caster_counter >= 120 {
                        self.caster_counter = 0;
                        let dx = pcx - cx;
                        let dy = pcy - cy;
                        let d = (dx * dx + dy * dy).sqrt().max(1.0);
                        self.vx = dx / d * 8.0;
                        self.vy = dy / d * 8.0;
                        self.ai[0] = 1.0;
                    }
                } else {
                    self.vx *= 0.95;
                    self.vy *= 0.95;
                    if self.vx.abs() + self.vy.abs() < 0.5 {
                        self.ai[0] = 0.0;
                    }
                }
            }
            NpcKind::GolemHeadFree => {
                // The free head: the spawn shield clears at 120 ticks
                // (the vanilla AI_048 clears `dontTakeDamage` — the
                // exact arm lands with the full machine), then the
                // EoC-shape hover.
                self.caster_counter += 1;
                if self.caster_counter > 120 {
                    self.dont_take_damage = false;
                }
                let tx = pcx - cx;
                let ty = (pcy - 150.0) - cy;
                let d = (tx * tx + ty * ty).sqrt().max(1.0);
                self.vx += (tx / d) * 0.07;
                self.vy += (ty / d) * 0.07;
                self.vx = self.vx.clamp(-7.0, 7.0);
                self.vy = self.vy.clamp(-7.0, 7.0);
            }
            _ => {}
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// The Creeper (`AI_055`, `NPC.cs:32657-32735`): the orbit around'''),
])
print('PART 1 DONE')
