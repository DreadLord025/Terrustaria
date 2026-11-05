# NPC AI taxonomy — the porting map (M4-NPC-2)

Extracted from the 1.4.5.8 decompile (`NPC.cs` SetDefaults' `if (this.type == N)`
chains, 243 pre-250 ids read; `tools/npc_taxonomy.py` regenerates
`tools/npc_table.txt` — the per-id `aiStyle/hp/dmg/def/kb` table this doc cites).
The port's movement archetypes live in `crates/tr-app/src/npc_ai.rs`
(`walker_step` = AI_003, `flyer_step` = AI_002's fly-and-bob).

## The archetype map

| aiStyle | Archetype | Port machine | Kinds (the vanilla count) |
| --- | --- | --- | --- |
| 1 | Slime hop | done (`SlimeVariant` + hop machine) | 27 |
| 2 | Eye fly-and-bob | done (`flyer_step`) | 13 |
| 3 | Ground walker/fighter | done (`walker_step`) | 165 — the biggest family |
| 5 | Hover + dash attacks | TODO `hover_step` | 18 |
| 6 | Worm segments | TODO `worm_step` (the head AI + the body follow) | 44 (the segment chains) |
| 7 | Town NPC walk | done (the Guide's wander subset) | 58 |
| 8 | Caster (teleport + shoot) | TODO `caster_step` | 13 |
| 14 | Bat fly (the erratic flight) | TODO (a flyer_step variant with the direction randomization) | 17 |
| 16 | Swim | TODO `swim_step` | 13 |

## The pre-hardmode candidates per follow-up batch

### Batch: aiStyle 3 walkers (the cave/surface layers)
| id | name | hp | dmg | def | kb | spawn layer |
| --- | --- | --- | --- | --- | --- | --- |
| 21 | Skeleton | 60 | 20 | 8 | 0.5 | the caves |
| 44 | Undead Miner | 70 | 22 | 9 | 0.5 | the caves |
| 31 | AngryBones | 80 | 26 | 8 | 0.8 | the dungeon |
| 26/27/28 | Goblin Peon/Thief/Warrior | 60/80/110 | 12/20/25 | 4/6/8 | 0.7-0.8 | the goblin invasion (M4-EVENT) |
| 52 | Doctor Bones | 500 | 20 | 10 | 0.5 | the jungle |
| 53 | The Groom | 200 | 14 | 8 | 0.5 | the blood moon |

### Batch: aiStyle 14 flyers (the bat family — the erratic-flight variant)
| id | name | hp | dmg | def | kb | spawn layer |
| --- | --- | --- | --- | --- | --- | --- |
| 49 | Cave Bat | 16 | 13 | 2 | 0.8 | the caves |
| 51 | Jungle Bat | 34 | 20 | 4 | 0.8 | the jungle |
| 60 | Hellbat | 46 | 35 | 8 | 0.8 | the underworld |
| 66 | Voodoo Demon | 140 | 32 | 8 | 0.8 | the underworld |
| 48 | Harpy | 100 | 25 | 8 | 0.6 | the sky (hardmode-gated out of the ambient) |

### Batch: aiStyle 5 hovers (the hover+dash archetype — needs `hover_step`)
| id | name | hp | dmg | def | kb | spawn layer |
| --- | --- | --- | --- | --- | --- | --- |
| 6 | Eater of Souls | 40 | 22 | 8 | 0.5 | the corruption |
| 42 | Hornet | 48 | 26 | 12 | 0.5 | the jungle (the stinger shooter) |
| 23 | Meteor Head | 26 | 40 | 6 | 0.4 | the meteor biome |

### Batch: aiStyle 8 casters (the teleport+shoot archetype — needs `caster_step`)
| id | name | hp | dmg | def | kb | note |
| --- | --- | --- | --- | --- | --- | --- |
| 45 | Tim | 200 | 20 | 4 | 0.6 | the caves; the rune hat drop |
| 24 | Fire Imp | 70 | 30 | 16 | 0.5 | the underworld; the flame projectile |
| 32 | Dark Caster | 50 | 20 | 2 | 0.6 | the dungeon |

### Batch: aiStyle 16 swimmers
| id | name | hp | dmg | def | kb | spawn layer |
| --- | --- | --- | --- | --- | --- | --- |
| 58 | Piranha | 30 | 25 | 2 | 0.5 | the jungle pools |
| 65 | Shark | 300 | 40 | 2 | 0.7 | the ocean |
| 55 | Goldfish | 5 | 0 | 0 | 0.5 | the surface pools (passive) |

### Batch: aiStyle 6 worms (the head/body/tail segment chains — the biggest engine lift)
| id | name | hp | dmg | def | note |
| --- | --- | --- | --- | --- | --- |
| 10-12 | Giant Worm | 30/seg | 8/4 | 0/4/6 | the cave worm (the simplest chain) |
| 13-15 | Eater of Worlds | 150/seg | 22/13 | 2/4/8 | the corruption boss (M4-BOSS) |
| 7-9 | Devourer | 100/seg | 31/16 | 2/6/10 | the dungeon worm |

## The framework notes

- The port's machines live in `crates/tr-app/src/npc_ai.rs`:
  `walker_step` (AI_003) and `flyer_step` (AI_002) with the per-kind
  `WalkerParams`/`FlyerParams` presets; the bats (aiStyle 14) port as a
  flyer variant with the direction randomization on the aiTimer.
- The `NpcKind` registration checklist (~14 match sites: to_u16/from_u16/
  size/max_hp/contact_damage/knock_back_resist/defense/value/find_frame/
  render_key/netglue/npc_buffs/render assets/bestiary) — see the Guide's
  port commit and the agents' briefs.
- The vanilla stats table: `tools/npc_table.txt` (regenerate with
  `python -I tools/npc_taxonomy.py` from the primary tree).
