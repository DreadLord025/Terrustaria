# 25 — NPC AI families (the `aiStyle` dispatch)

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/PORT_MAP/ANALYSIS

## Scope

The NPC AI family layer is the behaviour dispatcher of `Terraria/NPC.cs`: the
`AI()` method (95,930 lines total; `AI()` at `NPC.cs:19819-41132`) walks an
`if / else if (this.aiStyle == N)` chain of **128 arms (aiStyle 0..127)**,
each arm either inlining a machine or calling one of the 30 private
`AI_0xx_*` methods (`AI_001_Slimes` 60341, `AI_002_FloatingEye` 52274,
`AI_003_Fighters` 55856, `AI_005_EaterOfSouls` 50244, `AI_006_Worms` 50963,
`AI_007_TownEntities` 52965, `AI_026_Unicorns` 62366, `AI_037_Destroyer`
49756, `AI_045_Golem` 19505, `AI_047_GolemFist` 19231, `AI_065_Butterflies`
44896, `AI_077/078/079/081` 41157/41577/42084/42459, `AI_084_LunaticCultist`
64414, `AI_107_ImprovedWalkers` 62926, `AI_108_DivingFlyer` 65390,
`AI_109_DarkMage` 65791, `AI_110_Betsy` 61850, `AI_111_DD2LightningBug`
66188, `AI_120_HallowBoss` 45953, `AI_121_QueenSlime` 45202,
`AI_123_Deerclops` 44017, `AI_127_Pal` 42929, …). The families are large:
`aiStyle 3` alone is 4,485 lines with 260 `this.type ==` branches, `aiStyle 7`
is 2,598 lines with 69 branches, `aiStyle 1` is 1,509 lines, `aiStyle 6` is
1,311 lines, `aiStyle 107` is 1,487 lines.

On the Rust side the family layer is split in two:

| Rust file | Role |
| --- | --- |
| `crates/tr-app/src/npc_ai.rs` (769 lines) | the **six shared archetype steps**: `walker_step` (71), `flyer_step` (128), `hover_step` (199), `caster_cycle` (398) + `find_teleport_spot` (316), `sphere_step` (488), `swim_step` (586), plus the `line_clear` (559) `Collision.CanHit` stand-in and the `WalkerParams`/`FlyerParams`/`HoverParams`/`CasterParams`/`SwimParams` tables |
| `crates/tr-app/src/npc.rs` | `NpcKind` (182 variants, 31-686), the **per-kind machines** (`update_*`, 77 of them, 5839-12240), and the `match self.kind` dispatch inside `update_rng` (4103-4413) — the port's stand-in for `AI()`'s `aiStyle` chain |
| `crates/tr-app/src/world_sim/npcs.rs` | the `AI_006` worm chain pass (`update_worms_in`, 800) + the `AI_000_TransformBoundNPC` talk-transforms (300-370) |

Module 05 owns the NPC core (defaults, damage, net sync); this module owns
the **AI family layer**: which vanilla family each Rust kind belongs to, and
whether the port reproduces the family 1:1 or approximates it with a shared
archetype.

## Key entities

- `decompiled/Terraria/NPC.cs::NPC.AI` (19819-41132) — the vanilla dispatcher;
  128 `aiStyle` arms, one per family.
- `decompiled/Terraria/NPC.cs::NPC.AI_003_Fighters` (55856-60340) — the
  biggest family (155 vanilla types): the ground walkers/shooters.
- `crates/tr-app/src/npc.rs::Npc::update_rng` (4103) — the Rust dispatcher:
  the `match self.kind` (4114-4413) replaces the `aiStyle` chain.
- `crates/tr-app/src/npc_ai.rs::walker_step` (71) — the `AI_003` core
  (walk toward the player inside an aggro window, jump the one-tile ledge,
  integrate the gravity).
- `crates/tr-app/src/npc_ai.rs::hover_step` (199) — the `AI_005` fly core
  (the desired velocity = the normalised target vector × `max_speed`,
  approached by `accel`).
- `crates/tr-app/src/npc_ai.rs::caster_cycle` (398) — the `AI_008` cycle
  (the stand, the 500-counter, the shots at 100/200/300, the teleport at 650).
- `crates/tr-app/src/npc_ai.rs::sphere_step` (488) — the `AI_009` one-shot
  launch + straight flight.
- `crates/tr-app/src/npc_ai.rs::swim_step` (586) — the `AI_016` wet machine.
- `crates/tr-app/src/npc_ai.rs::flyer_step` (128) — the fly-and-bob core
  (documented as `AI_002_FloatingEye`, but its only caller is the bat machine).
- `crates/tr-app/src/npc.rs::NpcKind` (31-686) — the 182-kind roster; each
  variant's doc comment names the vanilla `NPCID` id and its `aiStyle`.
- `crates/tr-app/src/world_sim/npcs.rs::update_worms_in` (800) — the `AI_006`
  chain pass (assemble, trail, split, head machine).

## Dependencies

- **Calls into:** `tr_core::damage` (knockback), `tr_core::tiledata::TILE_SOLID`
  (the archetype collision), `crate::player::collides_rect` (the tile
  collision stand-in), `crate::npc_buffs`, `crate::projectile` (the hostile
  shot pool), `crate::loot`, `crate::app::frame::rand01` (the RNG stand-in),
  `crate::world_sim::*` (the deferred spawn requests).
- **Called by:** `crate::world_sim::step` → `update_npcs` → `npc_step` →
  `Npc::update_rng`; the renderer reads the frame/sprite state the machines
  write; the net glue snapshots the same fields.
- **Shared data structures:** `Npc` (the `ai[0..3]`-shaped slots are folded
  into per-kind fields — `caster_counter`, `cast_windup`, `ai[]`,
  `swim_phase`, `rotation`, `sprite_dir`, `worm_id`/`seg_index`); `NpcKind`
  is the dispatch key.

## The C# families, grouped

- **0** bound town NPCs (the stand + the talk transform) · **1** slimes (the
  hop, the water float, the item containment) · **2** floating eyes (fly and
  bob, the alpha phase) · **3** the fighter family (walk, jump, per-type
  velocity tables, the shooter sub-block) · **4** the Eye of Cthulhu (the
  842-line phase machine) · **5** the hover family (fly + the per-kind
  dash/attack arms) · **6** the worm chains (the head machine + the follower
  trail) · **7** the town NPCs (homes, sitting, idle anims, shimmer).
- **8** the casters (teleport + shoot) · **9** the spheres · **10** the cursed
  skull · **11/12** the Skeletron head + hands · **13** the plant traps ·
  **14** the bats · **15** the King Slime · **16** the swimmers ·
  **17-25** the 1.4.4 critter wave (the settlers, the jellyfish, the birds).
- **26** the unicorn/charger · **27/28/29** the Wall of Flesh + eye + Hungry ·
  **30/31** the Twins · **32-36** the Prime head + four limbs · **37** the
  Destroyer · **38** the Frost Legion trio (the throw/kick/jump) ·
  **39** the Solar Sroller · **40-44** the spider wall-crawl, the tentacle
  traps, the Queen Bee's neighbours · **45-48** the Golem family.
- **49-73** the 1.4.4 content wave (the ice/crystal/plant sentries, the
  goblin-style shooters, the Duke Fishron, the shimmer critters) ·
  **74-76** the martian drone/rider/saucer · **77-82** the Moon Lord family ·
  **83-93** the 1.4.5 content (the cultist, the mimics, the Duke Fishron's
  neighbours, the Flying Dutchman).
- **94-104** the lunar pillars and their shards, the Sentient Vine ·
  **105-110** the Old One's Army (the crystal, the lane gate, the walkers,
  the divers, the dark mage, Betsy) · **111-119** the 1.4.4 critters (the
  lightning bug, the fairy, the balloon, the dragonflies, the lady bugs,
  the water striders, the blood nautilus, the seahorses, the dandelion) ·
  **120-127** the Hallow bosses (the Queen Slime), the pirate ghost, the
  Deerclops, the slime chests/balloons, the statue mimics, the Pal.

## Port status — the coverage table

`C# behaviour` is one line per arm; `Rust equivalent` names the machine;
the verdict is **PORTED** (the family machine is reproduced), **PARTIAL**
(a machine exists but arms are missing), **APPROXIMATE** (a generic archetype
stands in for the family), **MISSING** (no Rust kind / no machine).

| aiStyle | C# behaviour | Rust equivalent | Verdict | Severity |
| --- | --- | --- | --- | --- |
| 0 | the bound-NPC stand; the talk transform | empty arms (npc.rs 4157-4162); the transform in `world_sim/npcs.rs` | PARTIAL | cosmetic |
| 1 | the slime hop, the water float, the item containment | `update_slime` (5839) + `SlimeVariant` | PARTIAL | behavioural |
| 2 | the eye fly-and-bob, the alpha phase | `update_eye` (5916) — its own copy of the flyer core | APPROXIMATE | behavioural |
| 3 | the fighter walker (4,485 lines, 260 type branches) | `walker_step` via `update_zombie`/`update_skeleton`/`update_goblin`; `update_archer`; `update_martian_shooter` | APPROXIMATE | behavioural |
| 4 | the Eye of Cthulhu phase machine (842 lines) | `update_eye_of_cthulhu` (6018) | PORTED | behavioural |
| 5 | the Eater of Souls hover (719 lines) | `hover_step` + `HoverParams` presets | PORTED (4 of 18 types) | behavioural |
| 6 | the worm chains (1,311 lines) | `world_sim/npcs.rs::update_worms_in` | PORTED (subset) | behavioural |
| 7 | the town NPC machine (2,598 lines) | `update_guide` (6389) — a wander only | APPROXIMATE | behavioural |
| 8 | the caster cycle (560 lines) | `caster_cycle` + `update_caster` (6526) | PORTED (4 of 13 types) | behavioural |
| 9 | the sphere launch (195 lines) | `sphere_step` + `update_sphere` (12176) | PORTED | cosmetic |
| 10 | the cursed skull | — | MISSING | behavioural |
| 11 | the Skeletron head (the hand spawn, the spin) | `update_skeletron` (6637) — an EoC-shaped machine | APPROXIMATE | behavioural |
| 12 | the Skeletron hand (the tether to the head) | `update_skeletron_hand` (11269) | PORTED | cosmetic |
| 13 | the plant traps / creepers | — | MISSING | behavioural |
| 14 | the bats (638 lines) | `update_cave_bat` (6466) + `flyer_step` | APPROXIMATE | behavioural |
| 15 | the King Slime (`AI_015_KingSlime`) | `update_king_slime` (6311) — "simplified" | PARTIAL | behavioural |
| 16 | the swimmers (530 lines) | `swim_step` + `update_swimmer` (12187) | PORTED (3 of 13 types) | behavioural |
| 17 | the settling flyer | — | MISSING | behavioural |
| 18 | the jellyfish | — | MISSING | behavioural |
| 19 | the drifter | — | MISSING | behavioural |
| 20 | the crawler | — | MISSING | behavioural |
| 21 | the ground slug | — | MISSING | behavioural |
| 22 | the hover/eye-laser family (10 types) | — | MISSING | behavioural |
| 23 | the ghosts (83, 84, 179) | — | MISSING | behavioural |
| 24 | the birds (the poop arm, the walk) | — | MISSING | behavioural |
| 25 | the demon-eye variants | — | MISSING | behavioural |
| 26 | the unicorn charger (`AI_026`) | `update_nebula_beast` (8407); `update_stardust_spider_small` | PARTIAL | behavioural |
| 27 | the Wall of Flesh (418 lines) | `update_wof` (6704) | PORTED (classic) | behavioural |
| 28 | the WoF mirror eye | `update_wof_eye` (11185) | PORTED | cosmetic |
| 29 | the Hungry (the leash, the damage tiers) | `update_hungry` (6891) | PORTED (classic tiers) | behavioural |
| 30 | Retinazer (hover + dash) | `update_twin` (10867) | PORTED | behavioural |
| 31 | Spazmatism (hover + dash) | `update_twin` (10867) | PORTED | behavioural |
| 32 | the Prime head (the hover/charge cycle) | `update_prime` (10759) | PORTED | behavioural |
| 33 | the Prime saw (the dock tether, the dash) | `update_prime_limb` (11021) | PORTED | behavioural |
| 34 | the Prime vice | `update_prime_limb` (11021) | PORTED | behavioural |
| 35 | the Prime cannon (the bombs) | `update_prime_limb` (11021) | PORTED | behavioural |
| 36 | the Prime laser (the deathray) | `update_prime_limb` (11021) | PORTED | behavioural |
| 37 | the Destroyer (`AI_037`) | the worm pass + the same-kind chain | PARTIAL | behavioural |
| 38 | the Frost Legion trio (the speeds 4/3.5/3, the throw/kick/jump) | `update_skeleton` (walker at 0.8) for all three | APPROXIMATE | behavioural |
| 39 | the Solar Sroller (the wind-up → ball → unroll) | `update_solar_sroller` (9351) | PORTED | cosmetic |
| 40 | the wall-crawling spiders | — | MISSING | behavioural |
| 41 | the sand crawlers | — | MISSING | behavioural |
| 42 | the burrower | — | MISSING | behavioural |
| 43 | the Queen Bee (the rage ramp, the stinger, the minions) | `update_queen_bee` (6566) — "v1" | PARTIAL | behavioural |
| 44 | the crystal shards | — | MISSING | behavioural |
| 45 | the Golem body (the charge/slam/flight) | `update_golem` (11545) | PORTED | behavioural |
| 46 | the Golem head (the tether, the fireballs, the beams) | `update_golem` (11545) | PORTED | behavioural |
| 47 | the Golem fists (the follow, the wind-up, the dash) | `update_golem` (11545) | PORTED | behavioural |
| 48 | the free Golem head (the volleys) | `update_golem` (11545) | PORTED | behavioural |
| 49 | the hover-charger | — | MISSING | behavioural |
| 50 | the sand sprites | — | MISSING | behavioural |
| 51 | the Plantera (the vines, the rage) | `update_plantera` (10934) — "v1" | PARTIAL | behavioural |
| 52 | the Plantera's creeper | — | MISSING | behavioural |
| 53 | the Plantera's tentacle | — | MISSING | behavioural |
| 54 | the Brain of Cthulhu (the shield, the transform) | `update_brain` (12013) | PARTIAL | behavioural |
| 55 | the Creeper (the orbit, the lunge) | `update_creeper` (11973) | PORTED | cosmetic |
| 56 | the gas sprite | — | MISSING | behavioural |
| 57 | the ice torchers | — | MISSING | behavioural |
| 58 | the flying Antlion | — | MISSING | behavioural |
| 59 | the Antlion | — | MISSING | behavioural |
| 60 | the goblin sentry (the day/night drift) | — | MISSING | behavioural |
| 61 | the sentry husk | — | MISSING | behavioural |
| 62 | the sentry spider | — | MISSING | behavioural |
| 63 | the granite elemental | — | MISSING | behavioural |
| 64 | the revenant | — | MISSING | behavioural |
| 65 | the butterflies (`AI_065`) | — | MISSING | cosmetic |
| 66 | the blood bats | — | MISSING | behavioural |
| 67 | the shimmer critters | — | MISSING | behavioural |
| 68 | the birds (the second set) | — | MISSING | behavioural |
| 69 | the Duke Fishron (`AI_069`) | — | MISSING | behavioural |
| 70 | the Vulture | — | MISSING | behavioural |
| 71 | the Crystal shard (the hover) | — | MISSING | behavioural |
| 72 | the tree worm | — | MISSING | behavioural |
| 73 | the hoppy | — | MISSING | behavioural |
| 74 | the charger (the drone + the Solar Corite) | `update_martian_drone` (7103), `update_solar_corite` (9427) | PORTED | behavioural |
| 75 | the mount riders (the seat link, the ride) | `update_saucer_mount` (7539), `update_scutlix_rider` (7419), `update_solar_drakomire_rider` (9327) | PARTIAL | behavioural |
| 76 | the Martian Saucer core (the scan, the gun) | `update_saucer_core` (10521) | PORTED | behavioural |
| 77 | the Moon Lord core | `update_moon_lord_core` (8636) | PORTED | behavioural |
| 78 | the Moon Lord hands | `update_moon_lord_hand` (8776) | PORTED | behavioural |
| 79 | the Moon Lord head | `update_moon_lord_head` (8922) | PORTED | behavioural |
| 80 | the Martian Probe (hover/detect/escape) | `update_martian_probe` (7000) | PORTED | behavioural |
| 81 | the True Eye of Cthulhu | `update_true_eye` (9018) | PORTED | behavioural |
| 82 | the Moon Leech Clot | `update_moon_leech_blob` (10348) | PORTED | cosmetic |
| 83 | the ritual spawn points | — | MISSING | behavioural |
| 84 | the Lunatic Cultist (`AI_084`) | — (the Rust `AncientCultist` runs the aiStyle-94 machine) | MISSING | behavioural |
| 86 | the sentries | — | MISSING | behavioural |
| 88 | the bronze/copper sentries | — | MISSING | behavioural |
| 89 | the hovering sentry | — | MISSING | behavioural |
| 90 | the sandninja | — | MISSING | behavioural |
| 91 | the tortoise | — | MISSING | behavioural |
| 92 | the desert prism | — | MISSING | behavioural |
| 93 | the Flying Dutchman (the crew, the drops) | `update_flying_dutchman` (7696) | PORTED | behavioural |
| 94 | the pillar ritual (the bob, the regen, the ritual) | `update_ancient_cultist` (10358) | PORTED | behavioural |
| 95 | the Stardust CellSmall growth | `update_stardust_cell` (8233) | PORTED | cosmetic |
| 96 | the Stardust Jellyfish (the bands, the shot) | `update_stardust_jellyfish` (8263) | PORTED | behavioural |
| 97 | the Nebula Floater (the teleport cycle) | `update_nebula_floater` (9562) | PORTED | behavioural |
| 98 | the dead arm (no vanilla type sets it) | — | MISSING (dead code) | cosmetic |
| 99 | the Stardust cell | — | MISSING | behavioural |
| 100 | the Nebula sphere | — | MISSING | behavioural |
| 101 | the Solar cell | — | MISSING | behavioural |
| 102 | the portal guardians | — | MISSING | behavioural |
| 103 | the Vortex sentries | — | MISSING | behavioural |
| 104 | the arm is `active = false` | — | MISSING | cosmetic |
| 105 | the Eternia Crystal (the shield, the win-death) | `update_eternia_crystal` (8200) | PORTED | behavioural |
| 106 | the DD2 lane gate (the spawn timer, the fade) | `update_dd2_lane` (8581) + the sim gate pass | PORTED | behavioural |
| 107 | the DD2 walkers (1,487 lines, 20 types) | `update_dd2_goblin` (7775), `update_dd2_bomber` (7833), `update_dd2_thrower` (10459), `update_dd2_skeleton` (10500), `update_dd2_kobold_walker` (10220), `update_ogre` (10252), `update_drakin` (10164), `update_wither_beast` (10413) | PARTIAL | behavioural |
| 108 | the DD2 diving flyers | `update_diving_flyer` (10011) | PORTED | behavioural |
| 109 | the DD2 Dark Mage (the summon/heal/shadowflame) | `update_dark_mage` (7889) | PORTED | behavioural |
| 110 | Betsy (the phases) | `update_betsy` (7983) | PORTED | behavioural |
| 111 | the DD2 Lightning Bug (`AI_111`) | — | MISSING | behavioural |
| 112 | the fairy critter (`AI_112`) | — | MISSING | cosmetic |
| 113 | the windy balloon (`AI_113`) | — | MISSING | cosmetic |
| 114 | the dragonflies (`AI_114`) | — | MISSING | cosmetic |
| 115 | the lady bugs (`AI_115`) | — | MISSING | cosmetic |
| 116 | the water striders (`AI_116`) | — | MISSING | behavioural |
| 117 | the blood nautilus (`AI_117`) | — | MISSING | behavioural |
| 118 | the seahorses (`AI_118`) | — | MISSING | behavioural |
| 119 | the dandelion (`AI_119`) | — | MISSING | cosmetic |
| 120 | the Hallow boss (`AI_120`) | — | MISSING | behavioural |
| 121 | the Queen Slime (`AI_121`) | — | MISSING | behavioural |
| 122 | the pirate ghost | — | MISSING | behavioural |
| 123 | the Deerclops (the 9-state machine) | `update_deerclops` (11336) | PORTED | behavioural |
| 125 | the clumsy slime balloon | — | MISSING | cosmetic |
| 126 | the statue mimic (`AI_126`) | — | MISSING | behavioural |
| 127 | the Pal (`AI_127`) | — | MISSING | behavioural |

**Totals:** 128 arms · **~60 have at least one Rust kind** · 68 arms have no
Rust kind (of which 1 — style 98 — is dead in the vanilla too) · of the ~60
covered arms, 24 are **PORTED** (a dedicated machine reproducing the family),
12 are **PARTIAL**, 5 are **APPROXIMATE** (a shared archetype stands in for
the family), 2 are **PARTIAL by omission** (0, 1).

## Archetype fidelity — 1:1 vs "a generic archetype stands in"

This is the most valuable distinction in the module: **only six vanilla
families have a shared Rust archetype**, and each is a *core* extraction, not
the family.

| Vanilla family | Rust archetype | Fidelity | Notes |
| --- | --- | --- | --- |
| `AI_003_Fighters` (155 types) | `walker_step` | core-only | the step reproduces the walk/jump/gravity; the family's 260 per-type branches (the velocity tables, the special arms, the 4 shooter sub-machines) are not in the archetype |
| `AI_005_EaterOfSouls` (18 types) | `hover_step` | core-only | the fly core is 1:1 (the desired-velocity + approach + the vertical damping); the per-kind dash/attack arms are per-kind |
| `AI_008` casters (13 types) | `caster_cycle` + `find_teleport_spot` | near-1:1 | the stand, the counter, the shot triggers, the teleport search and the shoot tick are reproduced; the flagged deviations are the `flag5` distance gate (constant-on) and the Dark Caster's dungeon-wall gate (off) |
| `AI_009` spheres (7 types) | `sphere_step` | 1:1 | the one-shot launch + the straight flight |
| `AI_016` swimmers (13 types) | `swim_step` | near-1:1 | the slope flip, the idle bob, the chase, the tilt, the tile integration; `Collision.CanHit` → `line_clear` |
| `AI_006_Worms` (43 types) | `world_sim/npcs.rs::update_worms_in` | near-1:1 | the chain assembly, the follower trail, the split, the head machine; the middle-split `Transform` is deferred |
| `AI_002_FloatingEye` (13 types) | `flyer_step` (**not used**) | duplicated | the archetype is documented as `AI_002` but `update_eye` (5916) re-implements the same maths inline; only the bat machine (`update_cave_bat`) calls `flyer_step` |
| `AI_001_Slimes` (26 types) | **none** | bespoke | `update_slime` is a standalone hop + water-tail machine, not an archetype |
| `AI_007_TownEntities` (52 types) | **none** | reduced | `update_guide` is a wander; the homes, the sitting, the idle anims, the shimmer and the teleport-to-home are absent |
| `AI_014` bats (17 types) | `flyer_step` + a hash jitter | approximate | the vanilla's velocity ramps and the `Main.rand` reroll are replaced |

Everything else is a **per-kind machine** ported from that style's own block
(the 30+ `update_*` functions in `npc.rs`), which is the right shape — but it
means the port's fidelity is per-kind, not per-family: a family with 155
vanilla types has 32 ported kinds and 123 unported ones.

## Tests

Static grep, no cargo run. `#[test]` counts: `npc.rs` 150, `npc_ai.rs` 2.
The archetype tests are `the_walker_chases_jumps_and_lands` and
`the_flyer_homes_and_bounces` (`npc_ai.rs:740`/`:758`). The per-kind machine
tests in `npc.rs` are `the_hover_kinds_spawn_fly_and_map`,
`the_hover_chases_the_target`, `the_cave_bat_spawn_flies_and_bites`,
`the_caster_teleports_and_shoots`, `the_sphere_launches_once_and_flies_straight`,
`the_swimmer_bobs_idly_in_water`, `the_worm_chain_assembles_and_trails`,
`the_worm_head_digs_and_stumbles`, `the_brain_teleport_dashes`,
`the_king_slime_releases_its_minion_batch`, `the_wof_eye_mirrors_and_volleys`,
`the_twin_machine_writes_the_hover_anchor`,
`the_eye_of_cthulhu_shares_the_twins_twenty_tick_beat`,
`the_wall_of_flesh_and_its_eye_share_the_mouth_cycle`.

**Coverage gap:** no test pins a family *identity* (i.e. "kind X runs the
aiStyle-N machine"), no test covers the frost trio's speeds, the goblin
speed, or the `aiStyle 2` alpha phase, and no test enumerates the
`aiStyle → kind` mapping — so a kind silently falling onto the wrong
archetype is invisible to the suite.

## Deviations & risks

Verified against the sources on 2026-11-12; rows ROADMAP marks DONE are
included and flagged.

1. **The Frost Legion trio runs the wrong speeds.** `npc.rs:4187-4189` sends
   `SnowmanGangsta | SnowBalla | MysterySnowman` to `update_skeleton`, whose
   `WalkerParams` are `speed: 0.8, jump: -6.2` (npc.rs:6441-6449). The
   vanilla `aiStyle 38` block (`NPC.cs:28995-29145`) uses `num551 = 4f`
   (`3f` for type 143, `3.5f` for type 145) and `num552 = 1f` (`0.7`/`0.8`),
   with the hop `velocity.Y = -8.2f` / `-6f` every third beat. The comment at
   npc.rs:4184-4186 claims "the per-type speeds 3/3.5/4 fold into the walker
   params" — **they do not**: all three share one speed. The vanilla's throw
   (type 143 → projectile 110 every 120 ticks), the kick (144) and the jump
   (145) are absent. Severity: **behavioural** (the whole invasion walks
   4-5× too slowly and never attacks at range). Not admitted.
2. **The `aiStyle 3` family collapses onto one walker.** All 32 ported
   aiStyle-3 kinds run `walker_step` with the *skeleton's* `speed: 0.8`,
   while the vanilla `AI_003_Fighters` carries per-type velocity tables and
   260 type branches (the martian/frost/pirate tables at `NPC.cs:59069+`).
   The comments at npc.rs:4175-4179 / 4184-4186 / 4190-4192 / 4364-4366
   repeatedly claim the per-type speeds "fold into the walker params"; they
   do not — only the goblins differ (1.0, via the shadowed `update_goblin`).
   Severity: **behavioural**, portfolio-wide. Not admitted.
3. **The goblin arm is still shadowed** (`npc.rs:4163-4165` runs
   `update_goblin` before the intended `update_skeleton` at 4180-4183), so
   the goblin army walks at 1.0 and the second arm is dead. Severity:
   **behavioural**, low-moderate. (Cross-ref module 05, deviation 4.)
4. **`GoblinSummoner` (the C# type 471) is an aiStyle-3 kind routed to the
   aiStyle-8 caster archetype.** The vanilla's type 471 sits in the
   `aiStyle 3` group (its summon is an `AI_003` arm); the port runs
   `update_caster` (npc.rs:4377), i.e. the teleport-and-shoot cycle.
   Severity: **behavioural**, moderate. Not admitted.
5. **`aiStyle 2` is duplicated instead of shared.** `npc_ai::flyer_step` is
   documented as `AI_002_FloatingEye`'s core (npc_ai.rs:125-128) but its only
   caller is `update_cave_bat`; `update_eye` (5916-5948) re-implements the
   same 0.12/3.0 fly-and-bob inline, and drops the vanilla's alpha phase
   (`ai[0] >= 300 → alpha 200`, `NPC.cs:52341-52355`) and its ±4 velocity
   ramps. Severity: **behavioural** + drift risk (two copies of one
   machine). Not admitted.
6. **The bats' velocity reroll is a positional hash, not `Main.rand`.**
   `update_cave_bat` (6472-6477) derives the jitter from `self.x`, so the
   reroll is deterministic per position rather than drawn from the shared
   stream every ~30 ticks as in `AI_014` (`NPC.cs:22849+`). Severity:
   **behavioural / determinism**. Not admitted.
7. **`aiStyle 7` — 52 vanilla types — is reduced to a wander.**
   `update_guide` (6389-6427) walks and flips every 240 ticks; the vanilla's
   `AI_007_TownEntities` (52965-55562, 69 type branches) owns the homes,
   the sitting, the idle animations, the shimmer and the
   teleport-to-home. The port has the wander + the arrival/shop machinery
   (module 29's scope) but not the NPC behaviour. Severity:
   **behavioural**. Not admitted (the family is not listed as ported).
8. **The aiStyle-84 cultist boss is missing and mislabelled.** The Rust
   `AncientCultist` kind's doc (npc.rs:367-374) claims `NPCID.CultistBoss`
   (439) but cites `NPC.cs:15172-15189` — the **type 422** row (130×270,
   aiStyle 94) — and its machine is the aiStyle-94 pillar ritual. The
   vanilla 439/440 are a 24×50 boss on `aiStyle 84` (`AI_084_LunaticCultist`,
   `NPC.cs:64414`), with no Rust equivalent. Severity: **behavioural** +
   identity (cross-ref modules 05 / 34).
9. **The `aiStyle` dispatch itself does not exist in Rust.** The port keys
   on `NpcKind` (182 variants) and has no `ai_style` field; the 128-arm
   family structure is implicit in the per-kind machines. A kind added later
   gets a hand-written machine rather than inheriting its family's default.
   Severity: **structural** (this is the module's headline deviation).
10. **`docs/NPC_AI_TAXONOMY.md` is stale.** Its archetype table still lists
    aiStyle 5/6/8/14/16 as `TODO` (`hover_step`, `worm_step`,
    `caster_step`, the bat flyer variant, `swim_step`) — all five exist now
    in `npc_ai.rs`/`world_sim/npcs.rs`. Severity: **cosmetic**, but it
    misdirects the next agent.
11. **ROADMAP §14's header is stale** ("Rust `npc.rs`, `world_sim/npcs.rs` ·
    🟡 3 kinds") — the roster is 182 kinds; and the summary table's
    "NPC families, town, bosses, events ✅" over-claims: the AI family layer
    covers ~60 of 128 vanilla families, and the two biggest (3 and 7) are
    approximations. Severity: **documentary**.

**ROADMAP DONE re-verification (the rows that claim NPC AI completeness):**

| ROADMAP row | Claim | Verdict |
| --- | --- | --- |
| `M4-NPC-2` (worms, aiStyle 6) | the chain framework is done | **PASS** — `update_worms_in` (800) reproduces the assembly, the trail, the split and the head machine; the middle-split `Transform` is a documented deferral |
| `M4-NPC-4` (hovers, aiStyle 5) | `hover_step` is the hover archetype | **PASS** for the core (the 4 ported kinds); the family has 14 more vanilla types, so the *family* is partial |
| `M4-NPC-5` (casters + spheres, aiStyle 8/9) | the caster/sphere archetypes | **PASS** with the two documented deviations (`flag5` constant-on, the Dark Caster's wall gate off); 9 of the 13 aiStyle-8 types are unported |
| `M4-NPC-6` (swimmers, aiStyle 16) | `swim_step` is the swim archetype | **PASS** with the `CanHit → line_clear` stand-in; 10 of the 13 types are unported |
| `M4-NPC-1` (umbrella) | "the slime colour family … the remaining families (fighters, worms, flyers…) continue as per-family branches" | **PASS** — honestly scoped; it does not claim family completeness |
| Summary table row "NPC families … ✅" | NPC AI is complete | **FAIL** — 68 of the 128 `aiStyle` arms have no Rust kind; aiStyle 3 and 7 are archetype approximations |
| `M4-NPC-3c/3h..3r` (display rows) | the FindFrame/display families | out of scope here (module 05 owns the display layer) |

## Task seeds

- **S — Fix the Frost Legion speeds.** Give `SnowmanGangsta`/`SnowBalla`/
  `MysterySnowman` their own params (`NPC.cs:28997-29012`'s `num551`/`num552`)
  or a small `update_frost_trio`; then add the type-143 throw (projectile 110
  every 120 ticks) and the type-145 jump. Depends on nothing.
- **S — Retire the misleading "folds into the walker params" comments**
  (npc.rs:4175-4179, 4184-4186, 4190-4192, 4364-4366) or make them true.
  The comments currently assert a porting step that does not exist.
- **S — Fix `docs/NPC_AI_TAXONOMY.md`** (the five stale `TODO` rows) and
  ROADMAP §14's "🟡 3 kinds" header.
- **S — Re-point `GoblinSummoner`** from `update_caster` (npc.rs:4377) to an
  `AI_003` summon arm, or document why the caster archetype is the intended
  substitute. Cross-ref module 05's shadowed-goblin arm.
- **M — A per-type speed table for the aiStyle-3 family.** Replace the single
  `speed: 0.8` with the vanilla's per-type `velocity.X` rows
  (`AI_003_Fighters`, `NPC.cs:59069+`), covering at least the martians
  (383/385/386/389/520), the frost trio, the pirates (212-216) and the
  goblins (26/27/28/111). Test: the walk speed per kind.
- **M — Deduplicate `aiStyle 2`.** Make `update_eye` call
  `npc_ai::flyer_step` (or fold `flyer_step`'s callers onto one function) and
  add the vanilla's alpha phase (`NPC.cs:52341-52355`).
- **M — Port the `aiStyle 84` cultist boss** (`AI_084_LunaticCultist`,
  `NPC.cs:64414`) and fix the `AncientCultist` identity (439 vs 422) —
  jointly with module 34.
- **M — A family-identity test.** Assert that each `NpcKind` maps to its
  vanilla `aiStyle` (a data table beside `NpcKind`), so a kind falling onto
  the wrong archetype fails the suite.
- **L — The town NPC family (`aiStyle 7`, 52 types).** The homes, the
  sitting, the idle animations and the shimmer; coordinate with module 29
  (housing/happiness) before starting.
- **L — The 68 unported arms**, grouped by content wave: the 1.4.4 critters
  (17-25, 40-44, 49-73, 86-92, 111-119), the 1.4.5 bosses (10, 13, 69, 120,
  121, 126, 127) and the lunar leftovers (83, 99-104). Each becomes either a
  new `npc_ai` archetype or a per-kind machine.
