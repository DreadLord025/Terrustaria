# 108-input-f42 — W4-f42 kind claims (input for the d37 roster matrix)

Status: CLAIMS INPUT — published by the W4-f42 lane
(`task/W4-f42-sandstorm-2`) for `docs/research/108-roster-master-reconcile.md`
(d37, latest read tip `a341daa12` R0-R9+E8). The f42 tip was NOT in the
E0 sweep nor the R re-reads (its kinds landed after those reads); this
file publishes the band so the matrix can adjudicate it. All rows
below re-derived from this branch's tree at the commit that carries
this file (see §6 hashes).

## 1. The nine-kind claim — protocol band 354-362

Base: `a725acb0` (origin/main at branch time, 270 kinds). The band was
picked as "next-free after 353 BlazingWheel" — the SAME independent
pick d37/d42/g8 made, so the whole band sits inside d37's canonical
354-399 corridor (see §3).

| Kind | vanilla `NPC.type` (cs_id == bestiary net_id) | proto `to_u16` | aiStyle | SetDefaults row (C#) |
|---|---|---|---|---|
| Tumbleweed | 546 | 354 | 26 | `NPC.cs:15974-15983` |
| Vulture | 61 | 355 | 17 | `NPC.cs:9556-9565` |
| Antlion | 69 | 356 | 19 | `NPC.cs:9668-9677` |
| Mummy | 78 | 357 | 3 | `NPC.cs:9799-9810` |
| DarkMummy | 79 | 358 | 3 | `NPC.cs:9812-9823` |
| BloodMummy | 630 | 359 | 3 | `NPC.cs:17107-17118` |
| LightMummy | 80 | 360 | 3 | `NPC.cs:9825-9836` |
| WalkingAntlion | 580 | 361 | 3 | `NPC.cs:14854-14865` |
| FlyingAntlion | 581 | 362 | 44 | `NPC.cs:14870-14879` |

Why port-local: the vanilla 61/69/78/79/80 collide with base's
GoblinSummoner/GrayGrunt/Deerclops/SkeletronHand/WallOfFleshEye protocol
slots (`npc/data.rs` `to_u16`); 546/580/581/630 are free but ride the
same band for one contiguous family block (the wyvern 223-228
renumber's precedent). The stat pin:
`w4_f42_the_desert_family_defaults_match_the_csharp_rows` (npc.rs).

## 2. Count deltas (this branch, vs base)

| Pin | base | f42 tip | note |
|---|---|---|---|
| `ALL_KINDS` | 270 | **279** | the nine appended last |
| `to_u16`/`from_u16` arms | 270 | **279** each | bijection, deny-unreachable |
| `s1_f_3` coverage pin | 270 | **279** | covered+sentinel walk |
| `CS_STYLES` rows | — | +9 | the aiStyle pin extends |
| bestiary `ENTRIES` | 249 | **258** | 9 own credits, net ids = vanilla |
| `BESTIARY_NPC_SHEET_IDS` | 181 | **190** | slots 181-189 appended |
| `STATIC_SHEET_NAMES` NPC rows | 270 | **279** | NPC_546/61/69/78/79/630/80/580/581 |
| `SFX_CUES` new names | — | **+6** | NPC_Hit_28/31/32, NPC_Killed_31/34/35; Hit_11/Hit_1/Killed_15/Killed_6 reused |

## 3. Collisions (protocol space, against the CURRENT origin tips)

All nine ids are contested. Hard (different-kind) squats on the band
(tips re-read at this commit's date; d42 `2d7ac08c3` has ALREADY
renumbered to vanilla-free per 108 E8, so its old 354-404 squat is
historical — shown for the record):

| id | f42 kind | d37 `a341daa12` (keeps 354-399) | g8 `82c93cec` | d42 old `9b22c091` (now moved) |
|---|---|---|---|---|
| 354 | Tumbleweed | CochinealBeetle | **Vulture (dup)** | DD2LightningBug → 578 |
| 355 | Vulture | CyanBeetle | BlueJellyfish | BartenderUnconscious → 579 |
| 356 | Antlion | LacBeetle | PinkJellyfish | WalkingAntlion → **580** |
| 357 | Mummy | SeaSnail | **Antlion (dup)** | FlyingAntlion (dropped, c4's) |
| 358 | DarkMummy | Squid | SpikeBall | LarvaeAntlion → 582 |
| 359 | BloodMummy | FlyingFish | Pixie | FairyCritterPink → 583 |
| 360 | LightMummy | FlyingSnake | Wraith | FairyCritterGreen → 584 |
| 361 | WalkingAntlion | Pirate | CursedHammer | FairyCritterBlue → 585 |
| 362 | FlyingAntlion | GoldfishWalker | EnchantedSword | ZombieMerman → 586 |

Same-kind duplicate ports of f42's kinds on OTHER lanes (merge dedupes,
E3 ownership rule: kinds-lane owns over wiring-lane; f42 is a wiring
lane, so every dup below resolves AGAINST f42's variant — f42 keeps its
AI/draw/sound/spawn wiring re-keyed on the canonical kind):

| Kind | f42 proto | canonical owner (current tips) | notes |
|---|---|---|---|
| Vulture | 355 | **d34@278** (`5f3435a87`; E3 keeps d34 over g8@354) | g8@354 drops |
| Antlion | 356 | g8@357 (E8 g8 residual band 745-761) | d34's 1-70 band does NOT port it |
| Mummy | 357 | d35@433 (`1afd327b9`; canonical renumber 698-730) | |
| DarkMummy | 358 | d35@434 → 698-730 | |
| LightMummy | 360 | d35@435 → 698-730 | |
| BloodMummy | 359 | **d42@630** (`2d7ac08c3`, vanilla-free landed) | |
| WalkingAntlion | 361 | **d42@580** (vanilla-free landed) | |
| FlyingAntlion | 362 | **c4@581** (`0df6964be` keeps it; d42's `49346dba6` DROPPED its 581 as "c4 style-44" — E3/E8's "keep d42@581 drop c4@581" is now INVERTED on the pushed tips; the matrix should re-adjudicate to keep c4) | |
| Tumbleweed | 354 | **f42 — UNIQUE across all re-read tips** | |

Tumbleweed is the only kind no kinds-lane ports. Its vanilla 546 is
free protocol space but sits inside d41's claimed band 491-560 (whose
claim already skips 541-545 for d33 and 547 for dd2-test). PROPOSAL for
the matrix: Tumbleweed keeps vanilla-free **546** (cs==proto, the
d38/d40 invariant) with a d41 skip row, and f42's other eight variants
dedupe to the owners above at merge — f42's spawner arms, stat/sound/
frame rows, sheets and portraits re-key onto the canonical kinds (the
E3 "dropped lane keeps its wiring" rule). f42 raises no objection to
losing 354-362; the band was never witness-pinned against a corridor
claim (d37's 354-399 predates it).

Note: d35's pushed Mummy/DarkMummy/LightMummy @433-435 ALSO collide with
d40's 430-461 corridor (108 E4: 433 Mummy↔ArmedZombieSlimed, 434
DarkMummy↔ArmedZombieSwamp, 435 LightMummy↔ArmedZombieTwiggy) — already
covered by the d35→698-730 block move; no new matrix edge.

Count-pin note: 108 R9 re-derives base ENTRIES as 241, but the pushed
base `a725acb04`'s `ENTRIES` array literal counts **249** rows
(scripted `BestiaryEntryDef {` count inside the array); f42's 258 =
249+9. The matrix's 486 union pin should re-check its base term.

## 4. The spawn-arm claim (not a kind claim)

`spawner.rs` ports `NPC.cs:3949-4018` (the sandstorm arm: the 8×9
`Spawning_SandstoneCheck` ≥40 scan `:5452-5491`, the pre-hardmode
546/61/69 ladder, the hardmode 541/510/542-545/78/79/630/80 ladder, the
546/580/581 fallthrough) AND `NPC.cs:4020-4035` (the post-sandstorm
mummy arms — hardmode, NO event gate: tile 53 `Next(3)`→78, tiles
112/234/116 `Next(2)`→79/630/80). Both terminal per C# else-if order;
witnesses `sandstorm_pre_hardmode_picks_the_desert_trio`,
`sandstorm_hardmode_picks_the_mummies_and_antlions`,
`post_sandstorm_mummy_arms_fire_without_the_event` (the last replays
the exact `Next(7)`+arm-roll draw order per seed).

## 5. 541/542-545 consume-arms — the d41/d33 read (this commit's date)

READ: `origin/task/W4-d41-npc-kinds-491-560` tip **`4ed22f0b3`** =
claim-only (`git diff-tree --name-only` empty). The d41 worktree's
UNCOMMITTED WIP adds only Salamander 498-506 — **no 541-545 rows
landed**. `origin/task/W4-d33-npc-ai-styles-95-127` tip **`b08acf463`**
carries `npc_ai_style102.rs` (`37c0ce77f`, the SandElemental 541,
`NPC.cs:40147-40542`) and `npc_ai_style103.rs` (`ee8b26681`, the
SandSharks 542-545, `NPC.cs:40543-40723`) — both **UNWIRED** (the
style73 precedent): no `NpcKind::SandElemental`/`SandShark` variants,
no 102/103 arms on the ai_style axis, modules keyed on a vanilla
`npc_type: i32` parameter (`style102_step(npc, npc_type, env)`).

CONSEQUENCE: f42's 541/542-545 arms stay consume-and-return-None (the
Wyvern shape); the `AnyNPCs(541)`/`CountNPCS(510)` gates read the
protocol-id roster where those vanilla ids are free slots — documented
in the arm. NO wiring this round.

ROUTING NOTE for the roster wave (kind-arms route via the ai_style
AXIS, not `update_kind_ai`): when the 541-545 kinds land, add
`102 => …style102_step(self, self.kind.cs_id() as i32, env)` /
`103 => …style103_step(…)` arms to the `match self.kind.ai_style()`
axis (`npc.rs:4752`, S1-D-4c one-live-arm-per-style) — NOT kind-match
arms (the axis claims the style first; a kind arm would be the retired
stand-in shape). The modules' `npc_type` parameter takes `cs_id()`
directly (541-545), so no module edit is needed. The `Style102Env`
(DustPool + SpawnCtx + rng + player candidates) rides the world_sim
dust pre-pass seam (the style-56 DungeonSpirit precedent). The 510
DuneSplicer is outside d33's 95-127 band and stays unported.

## 6. Witnesses + hashes (WRITTEN NOT RUN — code-only mode)

- `spawner.rs`: `sandstone_check_needs_forty_sand`,
  `sandstorm_pre_hardmode_picks_the_desert_trio`,
  `sandstorm_hardmode_picks_the_mummies_and_antlions` (borrow repair:
  the 112-row mutation now re-borrows the ctx),
  `post_sandstorm_mummy_arms_fire_without_the_event` (new).
- `npc.rs`: `w4_f42_the_desert_family_defaults_match_the_csharp_rows`
  (new, the §1 pin); `ALL_KINDS` 279 + the round-trip/`s1_f_3` pins
  re-pinned; `CS_STYLES` +9.
- `audio.rs`: the +6 SFX bank rows close the `s1_e_21c` gap the
  ALL_KINDS extension would otherwise open.
- Branch hashes: base `a725acb04`; phases `fb326084b` (Djinn curse),
  `4bf7b2e88` (sandStorm field), `75f055fcf` (the nine kinds + spawn
  arm); this file + the :4020-4035 tail + the pin repairs land in the
  commit that carries this file (see the branch tip).
