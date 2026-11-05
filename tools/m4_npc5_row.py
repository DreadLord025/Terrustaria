"""M4-NPC-5: the ROADMAP atlas row."""
import io
import os

os.chdir('C:/Users/DreadLord/Desktop/Terraria-Rust')

p = 'docs/ROADMAP.md'
s = io.open(p, encoding='utf-8').read()

old = '| M4-NPC-2 | `M4-NPC-2-worms` | multi-segment framework [14] |'
assert old in s, 'worms row anchor'
s = s.replace(old, '''| M4-NPC-2 | `M4-NPC-2-worms` | multi-segment framework [14] |
| ✅ M4-NPC-5 | `M4-NPC-5-casters` | the caster archetype (aiStyle 8) + the sphere archetype (aiStyle 9): `caster_cycle` in `npc_ai.rs` — the stand (`velocity.X *= 0.93` snapped ±0.1, `:20831-20835`), the 500 counter init (`:20836`), the shot triggers at `ai[0]` 100/200/300 (`ai[1] = 30`, `:21069`), the teleport attempt at 650 via `find_teleport_spot` — `AI_AttemptToFindTeleportSpot`'s 100-attempt random column scan (`:18937-19007`: the solid floor, the 4-row free body `i-4..i-1`, the lava gate, the ±5-tile telefrag rect), the placement `:20924-20930`, the generic gravity `:90465-90470`; the shoot at the windup tick (`num89` 25, the Fire Imp 10, `:21115`/`:21166`); `sphere_step` — the one-shot launch (`num120` 6, the Burning Sphere 5, `:21395-21429`), straight fly through tiles (`noGravity`/`noTileCollide`); the seven kinds — Goblin Sorcerer 29 (18×38, 40 HP), Tim 45 (200 HP, value 5000), Dark Caster 32 (50 HP), Fire Imp 24 (70 HP, def 16), Chaos Ball 30 (16×16, 1 HP), Burning Sphere 25, Water Sphere 33; Tim's ChaosBallTim 665 folds into ChaosBall (identical stats/name, `:17398`); the sphere spawns ride `wants_sphere` (the `wants_minion` shape) resolved in `world_sim/npcs.rs`; the spawner arms — the Tim deep-cavern roll `Next(200)` (`:5014`) and the Fire Imp underworld roll `Next(14)` (`:4891`), mutually exclusive by the vanilla depth gates, plus the live `spawn_npc_at` y-unit fix (tile → pixel at the boundary); the casters' pose-machine FindFrame (`:77019-77054`) and the Fire Imp's 10-frame machine (`:76979-77017`) | done 2026-11-05: 6 tests (the teleport/telefrag/shoot cycle, the Fire Imp tick, the sphere launch-once/straight-fly, the spawn/bestiary round-trip, the depth-band split); deviations — `flag5`'s firing-distance check constant-on (the MaxWorldViewSize rect lands with the camera work), the Dark Caster's dungeon-wall teleport gate constants-off |''')
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('ROADMAP row ok')
