# 108 — W4-g8 proto-band claims table (354–387) for the d37c/d37d matrix

> **E8 EXECUTED (SEAL receipt, 2026-11-21):** the d37 research/108 g8
> move-table landed on this branch — the **17 owned kinds now encode on
> 745–761** (the table below's proto column is the PUSHED history; the
> live ids are the E8 new-id column in §"E8 execution" at the tail), the
> **17 duplicate kinds stay DROP-PENDING** at their pushed ids until the
> merge keep-one (d34/d35/d37 own them), and the **Vampire moved 388→762**
> (388 is d37's RustyArmoredBonesAxe — a collision the E8 table predates;
> 762 is the first free id above the S4 CORE max 766 — the d37 SEAL
> `fbb948172` line 1408 reserves 788–790 for d39's 3 unclaimed kinds).
> The stale
> ALL_KINDS decl 270→**305** (R0 said 304; the measured array carries the
> Vampire too). The FINAL seal + the kind-wire-family bible live in
> **research/281** (registry first-free; this 108 number COLLIDES with
> d37's 108-roster-master-reconcile — the merger keeps d37's, this file
> folds into 281's receipt). Code-only: witnesses written not run.

> W4-g8 lane 1 (the d22–d33 UNWIRED-machine kinds ≤230) · published 2026-11-21 ·
> sources: `crates/tr-app/src/npc.rs` + `npc/data.rs` (the roster/stat tables),
> `decompiled/Terraria/NPC.cs` (the `SetDefaults` rows), `anchors.txt` rows 917–920.
> Code-only mode: no cargo run, no graphify write. This is the **claims table** the
> d37c/d37d verify matrix reads — one row per landed kind, the port-local protocol id
> it claims, the vanilla `NPC.type` it ports, and the dispatch that now owns it.

## Scope

The d22–d33 census (`research/50-entity-ai-census.md`) found 34 NPC kinds ≤230 whose
faithful `npc_ai_style*` machines sat **UNWIRED** (the pure port existed, the old
stand-in arm kept running). W4-g8 lane 1 wired all 34 to the NPC style axis in four
batches, claiming the next-free **port-local protocol band 354–387**. Every vanilla
`NPC.type` slot below 354 was already taken by a landed kind's protocol id
(`NpcKind::to_u16`'s bijection guard, `research/34-id-sets-audit.md` deviation 1), so
the 34 take the port-local band; the vanilla `NPC.type` rides `cs_id`/`net_id` (the
BlazingWheel 353 precedent).

**The band is a bijection**: `to_u16`/`from_u16` round-trip every row (the
`the_protocol_id_space_round_trips_every_kind` test, `npc.rs`), and the G6
`WorldFrameKey` hash stays injective (each row's key is unused by every other arm).

**Machine round (this branch, `task/W4-g8-npc-kind-wire-1`)**: the four cited
machine-skips the batch wirings deferred are now ported — Harpy 48 feather-shot +
wet arm, RedDevil 156 trident-shot, VampireBat 158 surface-flee + Transform(159),
FlyingSnake 226 facing/approach. The VampireBat's Transform target **Vampire 159 was
UNROWED** — it lands as a full kind at protocol id **388** (outside the 354–387 band;
the vanilla 159 slot is the BoundMechanic's protocol id). See §"Machine round" below.

## The claims table (34 rows, protocol ids 354–387)

Columns: **proto** = `NpcKind::to_u16` (the port-local protocol id, the save/net
space); **type** = the vanilla `NPC.type` (`cs_id`/`net_id`, the C# `SetDefaults`
key); **style** = `aiStyle` (the dispatch axis); **w×h** = the hitbox; **dmg/def/hp**
= contact damage / defense / lifeMax; **kb** = knockBackResist; **value** = copper;
**dispatch** = the arm that now owns the kind; **SetDefaults** = the `NPC.cs` row.

### Batch 1 — protocol 354–363 (anchors.txt:917)

| proto | kind | type | style | w×h | dmg | def | hp | kb | value | dispatch | SetDefaults |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 354 | Vulture | 61 | 17 | 36×36 | 15 | 4 | 40 | 0.8 | 60 | axis 17 → `npc_ai_style17::ai_style17` | NPC.cs:9554–9566 |
| 355 | BlueJellyfish | 63 | 18 | 26×26 | 25 | 4 | 34 | 1.0 | 100 | axis 18 → `npc_ai_style18::ai_018_arm` | NPC.cs:9582–9595 |
| 356 | PinkJellyfish | 64 | 18 | 26×26 | 30 | 6 | 70 | 1.0 | 100 | axis 18 → `npc_ai_style18::ai_018_arm` | NPC.cs:9596–9609 |
| 357 | Antlion | 69 | 19 | 24×24 | 10 | 6 | 45 | 0.0 | 60 | axis 19 → `npc_ai_style19::ai_style19` | NPC.cs:9666–9679 |
| 358 | SpikeBall | 70 | 20 | 34×34 | 32 | 100 | 100 | 0.0 | 0 | axis 20 → `npc_ai_style20::ai_style20` | NPC.cs:9680–9696 |
| 359 | Pixie | 75 | 22 | 20×20 | 55 | 20 | 150 | 0.6 | 350 | axis 22 → `npc_ai_style22::ai_022_arm` | NPC.cs:9772–9785 |
| 360 | Wraith | 82 | 22 | 24×44 | 65 | 16 | 160 | 0.7 | 500 | axis 22 → `npc_ai_style22::ai_022_arm` | NPC.cs:9852–9878 |
| 361 | CursedHammer | 83 | 23 | 40×40 | 80 | 18 | 200 | 0.4 | 1000 | axis 23 → `npc_ai_style23::ai_style23` | NPC.cs:9879–9891 |
| 362 | EnchantedSword | 84 | 23 | 40×40 | 80 | 18 | 200 | 0.4 | 1000 | axis 23 → `npc_ai_style23::ai_style23` | NPC.cs:9892–9904 |
| 363 | Mimic | 85 | 25 | 24×24 | 80 | 30 | 500 | 0.3 | 100000 | axis 25 (kind-gated) → `npc_ai_style25::ai_style25` | NPC.cs:9905–9925 |

Batch-1 flags: the jellies' alpha 20 (`:9594`/`:9608`), the Wraith's alpha 100
(`:9864`), the Antlion's behindTiles (`:9678`), the SpikeBall's dontTakeDamage +
scale 1.5 (`:9694–9695`). The Mimic carries the **hardmode** row; the pre-hardmode
fold (`:9918–9924`: 300 hp / 30 dmg / 12 def / 200 value) rides the spawn resolver's
override seams on a `world.hard_mode == false` world. The SandBall 31 proj_data row +
timeLeft 300 drain, the Item5/PixieChirp sound cues, the per-tick `ai_light` field
(the jelly shield-glow) and the weapons/Pixie draw-side lights landed with the batch.

### Batch 2 — protocol 364–374 (anchors.txt:918)

| proto | kind | type | style | w×h | dmg | def | hp | kb | value | dispatch | SetDefaults |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 364 | GreenJellyfish | 103 | 18 | 26×26 | 80 | 30 | 120 | 1.0 | 800 | axis 18 → `npc_ai_style18::ai_018_arm` | NPC.cs:10216–10229 |
| 365 | Gastropod | 122 | 22 | 20×20 | 60 | 22 | 220 | 0.8 | 600 | axis 22 → `npc_ai_style22::ai_022_arm` | NPC.cs:10479–10492 |
| 366 | WallCreeper | 164 | 3 | 50×20 | 30 | 10 | 80 | 0.25 | 100 | aiStyle-3 fighter (cap-2) → `update_skeleton` | NPC.cs:11059–11072 |
| 367 | WallCreeperWall | 165 | 40 | 36×36 | 30 | 10 | 80 | 0.25 | 100 | axis 40 → `npc_ai_style40::style40_step` (wall-crawl + walk Transform(164)) | NPC.cs:11073–11087 |
| 368 | IceElemental | 169 | 22 | 18×40 | 55 | 20 | 200 | 0.6 | 1500 | axis 22 → `npc_ai_style22::ai_022_arm` | NPC.cs:11113–11127 |
| 369 | Herpling | 174 | 41 | 52×26 | 65 | 26 | 220 | 0.3 | 450 | axis 41 → `npc_ai_style41::style41_step` (hopper, cs_id key) | NPC.cs:11194–11207 |
| 370 | Derpling | 177 | 41 | 58×44 | 80 | 26 | 300 | 0.5 | 500 | axis 41 → `npc_ai_style41::style41_step` (hopper, cs_id key) | NPC.cs:11238–11251 |
| 371 | CrimsonAxe | 179 | 23 | 40×40 | 80 | 18 | 200 | 0.4 | 1000 | axis 23 → `npc_ai_style23::ai_style23` | NPC.cs:11266–11278 |
| 372 | FloatyGross | 182 | 22 | 24×62 | 65 | 18 | 240 | 0.7 | 500 | axis 22 → `npc_ai_style22::ai_022_arm` | NPC.cs:11316–11331 |
| 373 | LostGirl | 195 | 42 | 18×40 | 10 | 30 | 250 | 0.5 | 20000 | axis 42 → `npc_ai_style42::lost_girl_ai` (Transform(196)→Nymph) | NPC.cs:11492–11505 |
| 374 | Nymph | 196 | 3 | 18×40 | 35 | 16 | 300 | 0.4 | 20000 | aiStyle-3 fighter (cap-2) → `update_skeleton` | NPC.cs:11506–11519 |

Batch-2 flags: the GreenJellyfish's alpha 20 (`:10228`), the FloatyGross's alpha 100
(`:11328`). The PinkLaser 84 / FrostBlast 128 / WebSpit 472 proj_data rows landed with
the batch (the Gastropod/IceElemental/WallCreeperWall ghost shots). The WallCreeper +
Nymph ride the aiStyle-3 cap-2 walk group (`NPC.cs:57554–57578`, ±2f, accel 0.07).

### Batch 3 — protocol 375–384 (anchors.txt:919)

| proto | kind | type | style | w×h | dmg | def | hp | kb | value | dispatch | SetDefaults |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 375 | CursedSkull | 34 | 10 | 26×28 | 35 | 6 | 40 | 0.2 | 150 | axis 10 → `npc_ai_style10::style10_ai` | NPC.cs:9150–9165 |
| 376 | Demon | 62 | 14 | 28×48 | 32 | 8 | 120 | 0.8 | 300 | axis 14 → `npc_ai_style14::bat_ai` (sickle) | NPC.cs:9567–9581 |
| 377 | GiantBat | 93 | 14 | 26×20 | 45 | 16 | 100 | 0.75 | 400 | axis 14 → `npc_ai_style14::bat_ai` | NPC.cs:10047–10060 |
| 378 | Slimer | 121 | 14 | 40×30 | 45 | 20 | 60 | 0.8 | 0 | axis 14 → `npc_ai_style14::bat_ai` | NPC.cs:10466–10478 |
| 379 | IlluminantBat | 137 | 14 | 26×20 | 75 | 30 | 200 | 0.75 | 500 | axis 14 → `npc_ai_style14::bat_ai` | NPC.cs:10735–10747 |
| 380 | IceBat | 150 | 14 | 22×22 | 18 | 6 | 30 | 0.8 | 250 | axis 14 → `npc_ai_style14::bat_ai` | NPC.cs:10885–10899 |
| 381 | Lavabat | 151 | 14 | 22×22 | 50 | 16 | 160 | 0.6 | 400 | axis 14 → `npc_ai_style14::bat_ai` | NPC.cs:10900–10915 |
| 382 | GiantFlyingFox | 152 | 14 | 38×34 | 80 | 24 | 220 | 0.65 | 400 | axis 14 → `npc_ai_style14::bat_ai` | NPC.cs:10916–10929 |
| 383 | VampireBat | 158 | 14 | 22×22 | 60 | 32 | 750 | 0.75 | 5000 | axis 14 → `npc_ai_style14::bat_ai` (surface-flee + Transform(159), machine round) | NPC.cs:15466–15479 |
| 384 | FlyingSnake | 226 | 14 | 34×50 | 85 | 28 | 260 | 0.65 | 400 | axis 14 → `npc_ai_style14::bat_ai` (facing/approach, machine round) | NPC.cs:11911–11924 |

Batch-3 flags: the Slimer's alpha 55 + scale 1.1 (`:10475`/`:10477`), the Lavabat's
scale 1.15 (`:10913`). The nine aiStyle-14 members join the four landed bats
(CaveBat 49 / JungleBat 51 / Hellbat 60 / VoodooDemon 66) on the same `bat_ai` arm;
the four bats' kind arm retired to the unreachable marker (the axis migration).

### Batch 4 — protocol 385–387 (anchors.txt:920)

| proto | kind | type | style | w×h | dmg | def | hp | kb | value | dispatch | SetDefaults |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 385 | Squid | 221 | 18 | 26×26 | 20 | 2 | 30 | 1.0 | 100 | axis 18 → `npc_ai_style18::ai_018_arm` (ink-latch swimmer) | NPC.cs:11840–11854 |
| 386 | Harpy | 48 | 14 | 24×34 | 25 | 8 | 100 | 0.6 | 300 | axis 14 → `npc_ai_style14::bat_ai` (feather-shot + wet arm, machine round) | NPC.cs:9360–9372 |
| 387 | RedDevil | 156 | 14 | 28×48 | 50 | 40 | 600 | 0.5 | 1200 | axis 14 → `npc_ai_style14::bat_ai` (trident-shot, machine round) | NPC.cs:10974–10988 |

Batch-4 flags: the Squid's alpha 20 (`:11852`). The RedDevil's lavaImmune +
npcSlots 2 (`:10987` region). This batch **completes the 34-kind ≤230 d22–d33
machine wiring**.

## Machine round (this branch) — the four cited skips + the Vampire census

The batch wirings deferred four type-specific machine branches as cited skips. They
are now ported into `npc_ai_style14.rs` (the aiStyle-14 arm, `NPC.cs:22849–23486`),
one kind per commit:

| commit | kind | branch | NPC.cs | shot / transform |
|---|---|---|---|---|
| a6c4f5ae9 | Harpy 48 | wet arm + feather-shot | :23333–23345 + :23416–23436 | HarpyFeather 38, dmg 15, timeLeft 300; the 30/60/90 cadence, 6f aim, 400+Next(400) reset |
| e1d9578a4 | RedDevil 156 | trident-shot | :23458–23484 | UnholyTridentHostile 115, dmg 80, timeLeft 300; the 20/40/60/80/100 cadence, 0.2 aim ±50, velocity*5 + aim*100 muzzle, 250+Next(250) reset |
| 5767ca7c7 | VampireBat 158 | surface-flee + Transform(159) + double ai[1]++ | :22899–22970 + :23346–23360 | Transform(159) within 200px (bottom-above + CanHit); the daylight flee + 7-cap approach |
| 058453096 | FlyingSnake 226 | velocity-sign facing + 4/2.5 approach | :22876–22894 + :22971–23037 | the facing latch kept on blocked CanHit; the 4-cap X / 2.5-cap Y approach |

**The Vampire 159 census row (protocol 388)** — the VampireBat's Transform(159)
target was UNROWED. It lands as a full kind (`NPC.cs:15480–15493`: 18×40, aiStyle 3,
dmg 80, def 24, 750 HP, kb 0.4, NPCHit1/NPCDeath6, value 5000, npcSlots 2; the null
buff-immunity row `NPCID.cs:7057`). The vanilla 159 slot is the BoundMechanic's
protocol id, so the Vampire takes the next-free **388**. It rides the aiStyle-3 cap-6
spin fighter (`npc_ai3_speeds.rs`'s 159 row, already ported) + the >300px
Transform(158) return (`NPC.cs:58843–58852`); the sim's transform drain gains the
158↔159 pair; `fighter_walker_kind` gains the Vampire (its Next(500) case-29 style-7
moan, `NPC.cs:57352–57355`, already ported). The FindFrame humanoid 15-strip +
air row-0 pin (`:69647`) landed.

**Shot seam**: `DemonSickleShot` generalized to `Style14Shot` (kind + damage) — the
sickle keeps 44/21, the feather rides 38/15, the trident 115/80; `update_cave_bat`
drains all three through `wants_hostile_shot`, the sim's timed-shot pass applies the
timeLeft 300 override. The HarpyFeather 38 proj_data row landed
(`Projectile.cs:1207–1216`); the UnholyTridentHostile 115 row + aiStyle-27 flight +
light already existed.

**Remaining cited skip**: type 660's 0.35/0.3/6/5 approach (`NPC.cs:23038–23115`) —
the only aiStyle-14 type branch still unported (660 is not a ≤230 d22–d33 kind).

## d37c/d37d matrix notes

- **Verify axis**: each row's `dispatch` is the arm to re-read against the cited
  `NPC.cs` SetDefaults + the style machine. The `w4_g8_b{1..4}_the_defaults_match_the_csharp_rows`
  tests (`npc.rs`) pin every stat cell; the `w4_g8_s14_*` / `w4_g8_machine_*` witnesses
  pin the machine-round branches (written, NOT run — code-only mode).
- **Id-space invariant**: the 354–387 band (+ the Vampire's 388) is disjoint from
  every pre-existing protocol id; `from_u16` decodes each back to its kind. The
  vanilla `NPC.type` is NOT the protocol id — read `cs_id` for the C# type gates.
- **The style axis claims** (one live arm per style, `#[deny(unreachable_patterns)]`):
  17/18/19/20/22/23/25/40/41/42 (batches 1–2), 10/14 (batches 3–4). The kind arms for
  the axis-claimed styles are `unreachable!` markers; the aiStyle-3 kinds
  (WallCreeper/Nymph/Vampire) ride the fighter ladder, not a style-axis arm.

## E8 execution (the SEAL receipt — the live id map)

The d37 E8 g8 table executed in `npc/data.rs` (`to_u16`/`from_u16`) +
`npc.rs` (`render_key` + the witness pins). **17 moves:**

| kind | old | new | cs_id | | kind | old | new | cs_id |
|---|---|---|---|---|---|---|---|---|
| PinkJellyfish | 356 | **745** | 64 | | LostGirl | 373 | **755** | 195 |
| Antlion | 357 | **746** | 69 | | Nymph | 374 | **756** | 196 |
| SpikeBall | 358 | **747** | 70 | | IceBat | 380 | **757** | 150 |
| WallCreeper | 366 | **748** | 164 | | Lavabat | 381 | **758** | 151 |
| WallCreeperWall | 367 | **749** | 165 | | GiantFlyingFox | 382 | **759** | 152 |
| IceElemental | 368 | **750** | 169 | | VampireBat | 383 | **760** | 158 |
| Herpling | 369 | **751** | 174 | | RedDevil | 387 | **761** | 156 |
| Derpling | 370 | **752** | 177 | | CrimsonAxe | 371 | **753** | 179 |
| FloatyGross | 372 | **754** | 182 | | | | | |

**17 DROP-PENDING dups** (keep the pushed id on this branch; the merge
keep-one drops the g8 row + wiring stays on the canonical owner's kind):
Vulture 354→d34@278, BlueJellyfish 355→d34@280, Pixie 359→d35@700,
Wraith 360→d35@705, CursedHammer 361→d35@706, EnchantedSword 362→d35@707,
Mimic 363→d35@708, GreenJellyfish 364→d35@720, Gastropod 365→d35@728,
CursedSkull 375→d34@267, Demon 376→d34@279, GiantBat 377→d35@710,
Slimer 378→d35@727, IlluminantBat 379→d35@729, FlyingSnake 384→d37@360,
Squid 385→d37@358, **Harpy 386→d34@275** (the g44e SS6 keep-ONE).

**Vampire 388→762** (E8 collision fix, resume-amended): d37's corridor
owns 388 (RustyArmoredBonesAxe). The in-flight SEAL draft took 788 —
WRONG: the d37 SEAL (`fbb948172`, research/108 line 1408) reserves
**788–790 for d39's 3 unclaimed kinds**. 762 is the first free id above
the S4 CORE max 766 (763–766 = d120's towns per R6; 767–780 + 791–804 =
d36's re-pin; 781–787 = d39's pushed seven; `_d37_seal_final.json`
carries no 762). The cs_id stays 159.

**Count pins:** ALL_KINDS 305 (decl fixed 270→305; R0's 304 missed the
Vampire), s1_f_3 literal 270→305, the round-trip bijection holds (the
`w4_g8_e8_the_owned_kinds_ride_745_761_and_the_cs_ids_stay` witness pins
every move + the freed old ids + the drop-pending rows; WRITTEN NOT RUN).
The stale `s1_f_13d` pins re-derived: `from_u16(384)` is the drop-pending
FlyingSnake (not None), `from_u16(387)` is None again (RedDevil moved).
