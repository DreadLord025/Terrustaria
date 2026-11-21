# 91 — FINAL: the fighter (aiStyle 3) census + the heart/star seam + the d22r reconcile receipt

W4-g4 branch `task/W4-g4-npc-style3-fighter`, FINAL phase (base DONE
`a9369ec3`, code commit `55840d9a`). Code-only lane: witnesses WRITTEN NOT
RUN (no cargo/rustc/rustfmt/tests/dotnet/graphify invoked). This document
closes the four FINAL items the Lead scoped: (1) the
`NPCLoot_DropHeals`/`DropCommonLifeAndMana` heart/star seam, (2) the
d22-live-session reconcile receipt, (3) the fighter-variant FINAL census,
(4) this research/91 FINAL.

## 1. The heart/star seam (item 1) — PORTED, f35 did NOT claim it

> **SEAL CORRECTION (research/91-seal-keep-one.md §1):** this section's
> ownership claim is INVERTED. The `:39` disclaimer below sits in f35's
> **"Port status BEFORE this task"** table — f35's pushed tip ported the
> SAME seam FIRST (`b72cdfbfc` + `00a5b27db`, both BEFORE g4's
> `55840d9a`) as a SUPERSET (the boss tail + Badger's Hat + EoW latch).
> Keep-one: **f35 owns the heal seam**; g4's duplicate `spawn_heal_drops`
> was RETIRED by the seal commit. The C# draw census below stands
> (verified line-by-line; both ports implement it identically) — read it
> as the audit trail for f35's `npc_death_heals.rs`.

**Probe result:** f35's tip `e0e5b53d` (origin/task/W4-f35-expert-drop-trees)
explicitly leaves the death-side heal seam OUT:
`docs/research/65-expert-drop-trees-census.md:39` —
"`NPCLoot_DropHeals`/`DropCommonLifeAndMana` | `NPC.cs:79098-79195`/
`:79531-79557` | — | missing, OUT of this task (same lane)". Its PORT_MAP
row + ROADMAP row both say "the death-side `DropBossPotionsAndHearts`/
`DropHeals` stay out (the death-events lane)". f35's 27 commits are the
boss-bag OPEN side + the crate/present openers — none touches
`NPCLoot_DropHeals`. **The seam is g4's to port** (the §6-seed cross-note
in the addendum named it f35's lane, but f35's own census disclaims it).

**Port (commit `55840d9a`):** `world_sim/npcs.rs::spawn_heal_drops` + the
two pure draw-order helpers `drop_common_life_and_mana` /
`drop_heals_type_switch`, wired LAST on the death path after
`spawn_coins`/`spawn_loot` (`NPC.cs:78897-78898` — money first, heals
after), on the `UpdateNPCs` stream.

### The C# census (`NPC.cs:79098-79195` + `:79531-79557`)

`NPCLoot_DropHeals` calls `NPCLoot_DropCommonLifeAndMana` FIRST, then the
per-type switch. The common block (`:79531-79557`):

* The `num` latch: `netID == 16` (MotherSlime) or `121` (Slimer) → `num = 1`
  (the else arm); `netID == 81` (CorruptSlime) → `num = 1`; every other
  kind → `num = 0`. Both `num == 1` paths skip BOTH rolls.
* The 1/6 heart-or-star block (`num == 0` only, `:79537-79547`):
  `RollLuck(6) == 0 && lifeMax > 1 && damage > 0`, then `Next(2) == 0 &&
  statMana < statManaMax2` → star 184, ELSE-IF `Next(2) == 0 && statLife <
  statLifeMax2` → heart 58. The second `Next(2)` draws ONLY when the first
  misses OR the mana gate fails (the else-if).
* The trailing 1/2 star roll (`num == 0` only, `:79553-79556`):
  `RollLuck(2) == 0 && lifeMax > 1 && damage > 0 && statMana <
  statManaMax2` → star 184.
* The `lifeMax > 1 && damage > 0` gates sit AFTER the `RollLuck` draws in
  the `&&` chain — a 1-hp critter still spends the luck rolls.

The per-type switch (`:79101-79194`), the heart-58 chances:

| C# line | types | chance | draws (luck 0) |
| --- | --- | --- | --- |
| `:79103-79119` | scarecrows 305-314 + Hellhound 329 / Poltergeist 330 | `RollLuck(4) == 0` | 1 |
| `:79120-79125` | Splinterling 326 | `RollLuck(6) == 0` | 1 |
| `:79126-79128` | HeadlessHorseman 315 | ALWAYS | 0 |
| `:79129-79137` | PresentMimic 341 | `Next(5,11)` hearts | 1 |
| `:79138-79145` | zombie elves 338-340 | `RollLuck(5) == 0` | 1 |
| `:79146-79151` | GingerbreadMan 342 | `Next(3) != 0` (2/3) | 1 |
| `:79152-79164` | MourningWood 325 / Pumpking 327 / Everscream 344 / IceQueen 345 / SantaNK1 346 | `Next(6)+6` hearts | 1 |
| `:79165-79170` | Creeper 267 | `Next(2)==0 && life below` | 1 |
| `:79171-79178` | EoW 13-15 | `Next(4)==0 && life below` | 1 |
| `:79179-79187` | TheHungryII 116 / leech 117-119 | `!expert \|\| Next(5)==0` | 0 classic / 1 expert |
| `:79188-79193` | Probe 139 | `Next(2)==0` | 1 |

**Per-difficulty census:** the ONLY difficulty input in either function is
the 116-119 leech arm's `!Main.expertMode || Next(5) == 0` (`:79183`) —
classic always drops (the `||` short-circuits the draw), expert/master
draw a 1/5 gate. Everything else is difficulty-free. The port is
single-difficulty classic (the `spawn_loot` `is_expert: false` precedent);
the expert arm is threaded for the difficulty system's landing.

**The seasonal remap:** each heart/star request rides
`Item.NewItem`'s seasonal remap (`Item.cs:49668-49714`, the ported
`seasonal_pickup_kind`) BEFORE the velocity triplet — the S1-P-i52 order
(the anniversary/both-seasons arms draw one `Next(len)` on the same
stream; the deterministic halloween/xmas remaps draw nothing).

### The spawn_coins stand-ins wired LIVE (the §6-seed fix)

`spawn_coins` carried two false stand-ins (`midas = false`,
`blood_moon = false`). Both are now LIVE:
* `npc.buffs.has_buff(72)` — `this.midas` (`NPC.cs:79631`), the Flask of
  Gold / Golden Bullet inflict (`melee_on_hit.rs:127`,
  `projectile_special_bullets.rs:442`). The midas `Next(30,51)` bump draws
  ONLY when the debuff is up.
* `app.world.blood_moon` — `Main.bloodMoon` (`NPC.cs:79660`). The
  blood-moon `Next(101)` bump draws ONLY when the flag is up.

A clean classic kill stays byte-identical to the pre-wiring stream (both
gates short-circuit). The witnesses pin the stream deltas.

**Witnesses (9, WRITTEN NOT RUN):** the common-block draw census (3 draws
at luck 0 all-zero), the num latch (16/121/81 draw nothing), the else-if
fall-through (mana closed → heart + trailing star), the critter
luck-rolls-before-stat-gates, the type-switch draw census (20 rows), the
leech difficulty gate (classic 0 draws / expert 1), the full-life gates
still draw, the midas/blood-moon stream deltas, the death-path stream
ride.

## 2. The d22-live-session reconcile receipt (item 2) — LANDED, fold-list CONSUMED

**Probe result:** the d22 tip has MOVED from `b24977d9` (the addendum's
§0 probe, which found the core "still pre-P5") to **`bcd42ddd1`** — the
**W4-d22r G4 reconcile** has landed, four commits on top of `b24977d9`:

* `0cfd08ac5` — reconcile phase 1: the graveyard bash gate signature
  union (P5) + the tall-gate drain (P4) + the blood-moon census correction.
* `4150d3748` — reconcile phase 2: the banner-buff strength lane (P1) +
  the summoner phase writes (P2) + the confused arms (P3).
* `e6c83bef8` — reconcile phase 3: the confused-immunity census (P9).
* `bcd42ddd1` — reconcile phase 4: the reconcile record (ANALYSIS) + the
  item-3/item-4 adjudication.

**The addendum's §0 "still pre-P5" probe is now STALE** — it was written
against `b24977d9`, before the reconcile landed. The fold-list it
published has been EXECUTED by d22r. Verification (read from `git show
origin/task/W4-d22-npc-ai-styles-3-6`, not the worktree):

| Fold-list item | d22r consumption | evidence |
| --- | --- | --- |
| §1.1 `target_zone_graveyard` field | KEPT | `npc.rs` `pub target_zone_graveyard: bool` + ctor false |
| §1.2 sim pre-pass | KEPT | `world_sim/npcs.rs` `n.target_zone_graveyard = zone_graveyard` |
| §1.3 scan_g rng signature | UNION | `fighter_obstacle_scan_g(npc, world, rng, flag8, hop_flag, num57, player_center)` |
| §1.4 flag25 line | REPLACED | `let flag25 = npc.target_zone_graveyard && rng.next_max(60) == 0;` |
| §1.5 discouraged chain term | KEPT | `fighters_not_discouraged` reads the pre-stamp; `update_zombie` carries `&& !self.target_zone_graveyard` |
| §1.6 module header | UPDATED | "RNG census (W4-g4 lane 15, folded by the W4-d22 reconcile)... The zone is NOW WIRED" |
| §1.7 P5 witnesses | KEPT | the four P5 pins + `w4_d22_the_core_threads_the_graveyard_bash_draw` |
| §2 P6 generic arms | KEPT | the walker/ladder buff-31 flip serves the non-claimed riders; the axis core's `target_closest` carries the flip (one flip only) |
| §3 P9 immunity rows | KEPT | `Skeleton \| UndeadMiner => &[20]`, `GoblinSummoner => &[31, 153]`, etc. — NOT re-folded to `&[]` |
| §4 GoblinPeon smash | KEPT | `world_sim/npcs.rs` `:60049-60056` type-26 KillTile arm |
| §4 blood-moon census | KEPT | the DOOR_BASH_TYPES note corrected (flag24 resets normal night, 489 accumulates every night) |
| P4 tall-gate drain | KEPT | `tr_core::edit::shift_tall_gate` + the drain's gate-388 arm |

**Receipt:** the keep-both contract is SATISFIED. d22's style-3 core is
canonical; every g4 P1-P6/P9 hunk survived its landing. The d22r ANALYSIS
record (`bcd42ddd1`) explicitly cites "g4's `research/91` +
`91-addendum-d22r-fold-list.md`" as the contract it executed. No g4 re-fold
is needed — the reconcile is complete.

**Draw-position note (unchanged):** the C# rolls the moans at the AI head
BEFORE the scan's bash draw; the port's moans ride the sim tail AFTER
`update_rng`. In a graveyard the bash `Next(60)` precedes the moan
`Next(1000)` — the S1-E-21e remainder, extended by one conditional draw.
d22's "RNG draw-free" claim for the core is now "draw-free OUTSIDE a
graveyard; one conditional `Next(60)` inside" (the d22r module header).

## 3. The fighter-variant FINAL census (item 3)

**The C# `AI_003_Fighters` (`NPC.cs:55856-60298`) type-branch census.**
Every `this.type == N` / `switch (this.type)` / `NPCID.Sets.X[this.type]`
read in the function, grouped by the port's routing:

### The plain-walker axis (d22's 22-claim set) — ALL routed

Skeleton 21, UndeadMiner 44, GoblinThief 27, GoblinWarrior 28, GoblinPeon
26, PirateDeckhand 212, PirateCaptainsMate 213, GrayGrunt 385, Scutlix
391, VortexSoldier 429, SolarSpearman 518, the scarecrow decade 305-314,
Splinterling 326. d22's `npc_ai_style3::claims` takes all 22; the dispatch
`3 if claims(kind) => fighter_core_step` routes them. The C# branches
that name these types:

* `:57606` the num83 group (21/26/385/429/... the 1.5 base + per-type rows)
  — the ladder (`npc_ai3_speeds.rs`).
* `:57554` the cap-2 group (27/326/...) — the ladder.
* `:57817-57880` the scarecrow num85 group (305-314, the hop machine
  305-309 + the plain accel 310-314) — the ladder.
* `:58203` the carrier group (391/427/415/419/518/532, the 5/0.25/0.7
  params) — the ladder. **The banner-carrier question resolves here:** the
  Scutlix 391 is the carrier; its params ride the ladder. The g4 worktree's
  old `update_scutlix` carried an INVENTED 50-px stop (`:58291-58304` is
  the 430-436/494/495/591 charge family's, NOT 391's — the C# 391 has no
  stop). d22r RETIRED it (`NpcKind::Scutlix => unreachable!("aiStyle 3's
  plain walkers ride the style axis")`). The g4 worktree's `update_scutlix`
  still carries the invented stop, but the axis claim supersedes it on the
  d22 reconcile — the stop is dead code there.
* `:59996` flag24 door-bash list (21/27 is NOT in it; 26 is NOT in it) —
  the scan. The GoblinThief 27 +1 (`:60027`) and the GoblinPeon 26 smash
  (`:60049`) are the g4 FINAL additions (commit `55840d9a`).
* `:60205` the charge-hop list (385 is in it; 389 is in it) — d22's
  `charge_hop_type`. The GrayGrunt 385 charge-hop is live on the axis.

### The zombie family (17 kinds) — `update_zombie`, ALL routed

Zombie 3, DoctorBones 52, TheGroom 53, BaldZombie 132, ZombieEskimo 161,
PincushionZombie 186, SlimedZombie 187, SwampZombie 188, TwiggyZombie 189,
FemaleZombie 200, ZombieRaincoat 223, ZombieDoctor 319, ZombieSuperman
320, ZombiePixie 321, ZombieXmas 331, ZombieSweater 332, BloodZombie 489.
The C# branches: `:57340` the moan 1000-roll, `:58322-58401` the num107
default arm (the per-type caps + the 489 distance-scaled row), `:59996`
flag24 (the zombie family resets normal night), `:60007` flag27. The
BloodZombie 489 is NOT in flag24 — it accumulates every night (the
blood-moon census correction). **MossZombie 691 and ZombieMerman 586 are
NOT ported** (no bestiary rows) — kind-roster gaps, not AI-arm gaps.

### The shooters — their machines, ALL routed

GoblinArcher 111 (`update_archer`), PirateArcher 214 / PirateCorsair 215 /
PirateCaptain 216 (`update_martian_shooter`), BrainScrambler 381 /
RayGunner 382 (`update_martian_shooter`), StardustSoldier 411
(`update_stardust_soldier`), StardustSpiderBig 409
(`update_stardust_spider_big`), Predictor 424 (`update_nebula_predictor`),
VortexHunter 426 (`update_vortex_hunter`), ElfArcher 350
(`npc_ai_frost_mob::frost_mob_step`). The C# shooter block
(`:59069-59801`) names all of these. The per-type shot params
(num162 cooldown / num164 speed / num169 damage / num170 proj) are in each
machine. The confused resets (`:59197`) are the P3 fold.

### The martian head-arm kinds — their fns + the P3 resets

MartianOfficer 383 (`update_skeleton` + `martian_officer_bubble`, the
`:56222-56251` ForceBubble lifecycle), MartianEngineer 386 / GigaZapper
389 (`update_skeleton` + the `ai[2] = -60` confused reset, `:58986`/
`:59048`). The engineer's turret-spawn cycle (`:59004-59038`) and the
zapper's spear shot (`:59062-59065`) are unported (the shot arms land with
the projectile work) — the confused reset is the carried remainder.

### The pillar mobs — their machines

SolarDrakomire 415 (`update_solar_drakomire`, the `:56786-56853` rider
spawn), SolarSelenian 419 (`update_solar_selenian`, the `:56719-56784`
reflect leap), VortexRifleman 425 (`update_vortex_rifleman`, the
`:58463-58620` leap + the four confused arms), VortexAlien 427
(`update_vortex_alien`, the `:58622-58710` dive + the 1200-tick
Transform(426)), VortexHeadcrab 428 (`update_vortex_headcrab`, the
`:56855-56884` 300-tick Transform(427) + the roll).

### The summoner — its phase machine

GoblinSummoner 471 (`update_goblin_summoner`, the `:56491-56717`
shadowflame phases). The P2 phase writes (flight kb0/def28, ground
kb0.35/def26, pose kb0/def36) are the d22r fold.

### The frost siblings — `npc_ai_frost_mob` / `update_skeleton`

ZombieElf 338 / ZombieElfBeard 339 / ZombieElfGirl 340 / GingerbreadMan
342 / Yeti 343 / Krampus 351 (`update_skeleton`), Nutcracker 348 /
NutcrackerSpinning 349 / ElfArcher 350 (`npc_ai_frost_mob`). The C#
`:57637-57648` num83 rows + the `:57554` cap-2 group.

### The ONE genuine remainder: MartianWalker 520

**The MartianWalker 520 rides `update_skeleton` (the plain walker) in BOTH
g4 and d22, but the C# 520 is a SHOOTER-BLOCK type** (`:59069` list). Its
C# shot params: num162 = 15 (`:59109`), num164 = 8 (`:59253`), num170 =
592 / num169 = 35 (`:59442-59445`), the walk num180 = 4 / num181 = 1 /
num182 = 0.7 (`:59735-59739`), the `localAI[2]`/`localAI[3]` aim cycle
(`:59792-59799`), the `CanHitLine` top+20 aim (`:59637-59639`), the
400-px flag22 stop (`:59763-59766`). The port's `update_skeleton` gives it
the plain-walker walk (the 4.0 cap from `ai3_walk_speed`) but NO shot
machine, NO aim cycle, NO stop. **This is a shooter-machine gap, not an
AI_003 type-branch gap** — the 520's walk IS routed (the plain walker),
but its offence (the 592 laser) is unported. d22's census counts it as
"routed" (the walk), which is true for the movement but not the shot. The
shot belongs to the projectile-work lane (the same lane as the engineer's
turret + the zapper's spear). **Recorded as the one fighter shooter
remainder.**

> **SEAL CORRECTION (research/91-seal-keep-one.md §3):** the 520's lane
> is **d41** (band 491-560, claim `4ed22f0b3`), NOT a generic
> projectile-work lane and NOT d43 (631-697). d41's batch 2
> (`5d036721d`) set the precedent: it ported the salamander shooter
> machine AND its SalamanderSpit 572 projectile in-lane. The 520
> shooter-block (592 laser + aim cycle + 400-px stop, the params above
> re-verified at the seal) is a **d41 task seed**. Only the engineer's
> turret + the zapper's spear (kinds 386/389, outside the band claims)
> stay with the projectile-work lane.

### The banner-carrier / goblin-peasant questions (the Lead's item 3)

* **Banner-carrier:** the C# has no "banner-carrier" AI_003 type. The
  banner carriers are the goblin invasion's standard-bearers, which in
  1.4.5.8 are the Scutlix 391 (the Martian carrier) — its params ride the
  carrier group ladder (`:58203`). The g4 worktree's invented 50-px stop
  is retired by d22r. NO missing AI_003 arm.
* **Goblin-peasant:** the C# has no "peasant" type. The goblin invasion
  four are GoblinPeon 26 / GoblinThief 27 / GoblinWarrior 28 /
  GoblinArcher 111 — all routed (the Peon on the axis + the smash drain,
  the Thief/Warrior on the axis, the Archer on `update_archer`). The
  "peasant" is the Peon (the door-smasher). NO missing AI_003 arm.

### The unported style-3 types — kind-roster gaps, NOT AI-arm gaps

The C# `SetDefaults` aiStyle-3 roster is ~191 types; ~68 have port kinds.
The ~123 unported types (the mummies 78-80/630, the AngryBones family
31/294-296, the torch zombies 590/591, the gnomes 624/631, the
CreatureFromTheDeep 461, the SwampThing 166, the Psycho 466, the Butcher
460, the BoneLee 287, the Nymph 196, the Werewolf 104, the ArmoredSkeleton
77, the desert ghouls 524-527, the DD2-adjacent 494-506, the moss zombie
691, the zombie merman 586, ...) are KIND-ROSTER gaps (the d34-d43 lanes),
not AI-arm gaps. The AI_003 branches that name them (the gnome statue
transform `:55863`, the psycho stealth `:55901`, the swamp thing hide
`:55938`, the merman wet swim `:56055`, the butchers' rage `:58070`, the
bone lee dash `:57944`, the charge family `:58262`, the chaos elemental
teleport `:60285`, ...) wait on their kinds' landing. The g4 FINAL commit
keeps the AngryBones +6 bash arm (`:60031`) and the Butcher always-open
(`:60042`) verbatim in the scan for their landing.

## 4. The research/91 FINAL (item 4)

This document is the FINAL research/91. It supersedes the addendum's §0
probe (the d22 tip has moved, the reconcile has landed) and closes the
four FINAL items. The INDEX row 91-F points here. The ANALYSIS residual
section gains the FINAL receipt (the heart/star seam ported, the d22r
reconcile consumed, the MartianWalker shooter remainder).

**Verification.** Code-only: every C# site re-read line-by-line
(`NPC.cs:55856-60298` the whole AI_003_Fighters, `:60300-60339` the two
despawn helpers, `:79098-79195`/`:79531-79557` the heal seam,
`:79619-79728` the money seam); the d22 tip read from `git show` (the
reconcile commits `0cfd08ac5`/`4150d3748`/`e6c83bef8`/`bcd42ddd1`); the
port seams checked by read/grep. The witness tests (9 heal-seam + 2
fighter-scan) WRITTEN, NOT RUN.
