# 68 — FindFrame wire-census (W4-d44 merger assist, code-only)

Audit date: 2026-11-21. Branch `task/W4-d44-findframe-low-1`, worktree
`TR-W4-d44`, base `bc9c0fe7`. **Code-only mode: no cargo/rustc/tests/graphify
invoked** — every claim below is git/file-read evidence against the PUSHED
remote tips (`git ls-remote origin` + `git show origin/<branch>:<file>`),
re-probed at census time; all twelve findframe branches are pushed and their
local worktree tips match the remote byte-for-byte. No code was changed on
any lane; this document is the only artefact.

Purpose: the merger's worklist for the ported-but-UNWIRED `NPC.FindFrame`
pool (the f27 census arithmetic: **602** top-level C# case ids − **248**
already port-covered on main − **75** d44–d49 claims = 279 remaining; minus
the **30** e13/e14-declared = **249 unclaimed**; f26/f27 took 15 each →
**219-kind unclaimed pool** at the f27 census; f26 slice 5 takes 15 more →
**204** after it).

## 1. Branch census (all pushed tips, probed 2026-11-21)

| lane | branch (`task/W4-…`) | pushed tip | module(s) | kinds | hook on its OWN branch |
| --- | --- | --- | --- | --- | --- |
| d44 | `d44-findframe-low-1` | `bc9c0fe7` | `npc_findframe_low1.rs` | 12 | NO — wired by g6 (`5113ff18`) |
| d45 | `d45-findframe-low-2` | `aea0aebd` | `npc_findframe_low2.rs` | 12 | YES (cs_id hook + `find_frame_wave_low2`, self-wired) |
| d46 | `d46-findframe-mid-1` | `055fcf8c` | `npc_findframe_mid1.rs` | 12 | YES (cs_id hook + `find_frame_wave_mid1`, self-wired) |
| d47 | `d47-findframe-mid-2` | `3d36f76c` | `npc_findframe_mid2.rs` | 12 | NO — wired by g6 (`2fe567f9`) |
| d48 | `d48-findframe-high-1` | `3eb2ee08` | `npc_findframe_c.rs` | 12 | NO on its own branch — **WIRED(d44)** on `task/W4-d44-findframe-low-1` (§7) |
| d49 | `d49-findframe-high-2` | `dadfd828` | `npc_findframe_high2.rs` | 12 / 13 ids | NO — wired by g7 (`b6c9c05c`+`1b273055`) |
| e13 | `e13-findframe-extra-1` | `9fbca588` | `npc_findframe_extra1.rs` | 15 | YES (self-wired) — **fully twinned**, keep-one drop candidate (§3) |
| e14 | `e14-findframe-extra-2` | `63bc2c9f` | `npc_findframe_extra2.rs` | 15 | NO — wired by g6 (`b58d8d2f`, 15-id mapper) AND g7 (`fbee59b3`, trimmed 7-id mapper) |
| f26 | `f26-findframe-extra-3` | `73150c56` | `npc_findframe_extra3.rs` (slice 3) + `npc_findframe_extra5.rs` (slice 5) | 15 + 15 | slice 3: NO — wired by g6 (`bf99e983`) AND g7 (`cb68161c`+`76b5252a`); slice 5: **WIRED(d44)** on `task/W4-d44-findframe-low-1` (§7) |
| f27 | `f27-findframe-extra-4` | `ad79bbb7` | `npc_findframe_extra4.rs` | 15 | NO — wired by g7 (`ba1319a8`+`63b66a4e`) |
| g6 | `g6-npc-findframe-wire-1` | `bf99e983` | wires low1 + mid2 + extra2 + extra3 | 54 ids / 4 hooks | the wire lane itself |
| g7 | `g7-npc-findframe-wire-2` | `6ca3e668` | wires extra2(trim) + extra3 + extra4 + high2 | 50 ids / 4 hooks | the wire lane itself |

In-flight (NOT pushed, probed via `git ls-remote origin` — no `f26c`/`f27b`
refs exist): **f26c slice 6** and **f27b slice 7**. Their kinds are unknown
until push; re-run this census after they land. The f26 slice-5 ANALYSIS
note pins slice 6's head: "269..280 case list — 278/279/280 ride the next
slice".

Main base for both wire lanes: `a725acb0` (wave-3 land). g6 contains the
d44 commits under rebased hashes (`2232e22c`/`0c7abbf0`/`e4e31dad`/
`fc261a3b` ≡ d44's `0a722ae0`/`1f838803`/`8b28f2e6`/`bc9c0fe7`); g7
cherry-picks the e14/f26/f27/d49 port commits. All hooks are `cs_id`-keyed
at the `npc.rs::find_frame` head, ahead of the body match, each `return`ing
after one machine tick; every hook is stream-inert today (zero collisions
over the `npc/data.rs` hand cs_id rows + the bestiary net_id fallback, per
the g6/g7 ANALYSIS inertness proofs and their `ALL_KINDS` sweep witnesses).

## 2. Per-kind table

Status legend: **WIRED(g6)** / **WIRED(g7)** = a pushed wire-lane hook claims
the id; **WIRED(d44)** = this branch's wire round (§7) claims it;
**branch-wired** = the port branch carries its own hook (d45/d46/e13);
**UNWIRED** = no pushed branch hooks the id. "twin" = the id is claimed by
two modules whose arms were verified verbatim identical (keep-one at merge).

### d44 low-1 — `npc_findframe_low1` (hooked only by g6)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 37 | OldMan | 21 | d44 | WIRED(g6) |
| 43 | ManEater | 3 | d44 | WIRED(g6) |
| 46 | Bunny | 7 | d44 | WIRED(g6) |
| 47 | CorruptBunny | 7 | d44 | WIRED(g6) |
| 48 | Harpy | 6 | d44 | WIRED(g6) |
| 56 | Snatcher | 3 | d44 | WIRED(g6) |
| 57 | CorruptGoldfish | 6 | d44 | WIRED(g6) |
| 61 | Vulture | 6 | d44 | WIRED(g6) |
| 62 | Demon | 5 | d44 | WIRED(g6) |
| 63 | BlueJellyfish | 7 | d44 | WIRED(g6) |
| 64 | PinkJellyfish | 7 | d44 | WIRED(g6) |
| 67 | Crab | 8 | d44 | WIRED(g6) |

d44 self-check (the "wire your own lane" clause): all 12 d44 kinds ride the
g6 hook `5113ff18`, which sits on top of the d44 commits themselves — the
module bytes on g6 are the d44 module. No d44-side wire needed; adding a
duplicate hook here would only create a merge conflict.

### d45 low-2 — `npc_findframe_low2` (branch-wired)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 69 | Antlion | 5 | d45 | branch-wired(d45) |
| 73 | GoblinScout | 16 | d45 | branch-wired(d45) |
| 74 | Bird | 5 | d45 | branch-wired(d45) |
| 83 | CursedHammer | 6 | d45 | branch-wired(d45) |
| 84 | EnchantedSword | 6 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims — dedupe §3 |
| 93 | GiantBat | 4 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |
| 94 | Corruptor | 3 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |
| 101 | Clinger | 5 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |
| 102 | AnglerFish | 6 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |
| 103 | GreenJellyfish | 7 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |
| 116 | TheHungryII | 6 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |
| 120 | ChaosElemental | 15 | d45 + e14 twin | branch-wired(d45); g6 extra2 also claims |

### d46 mid-1 — `npc_findframe_mid1` (branch-wired)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 150 | IceBat | 4 | d46 | branch-wired(d46) |
| 151 | Lavabat | 4 | d46 | branch-wired(d46) |
| 152 | GiantFlyingFox | 4 | d46 | branch-wired(d46) |
| 156 | RedDevil | 5 | d46 | branch-wired(d46) |
| 157 | Arapaima | 6 | d46 | branch-wired(d46) |
| 158 | VampireBat | 4 | d46 | branch-wired(d46) |
| 159 | Vampire | 15 | d46 | branch-wired(d46) |
| 160 | Truffle | 23 | d46 | branch-wired(d46) |
| 162 | Frankenstein | 15 | d46 | branch-wired(d46) |
| 163 | BlackRecluse | 8 | d46 | branch-wired(d46) |
| 164 | WallCreeper | 5 | d46 | branch-wired(d46) |
| 165 | WallCreeperWall | 4 | d46 | branch-wired(d46) |

### d47 mid-2 — `npc_findframe_mid2` (hooked only by g6)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 166 | SwampThing | 13 | d47 | WIRED(g6) |
| 168 | CorruptPenguin | 12 | d47 | WIRED(g6) |
| 169 | IceElemental | 4 | d47 | WIRED(g6) |
| 170 | PigronCorruption | 14 | d47 | WIRED(g6) |
| 171 | PigronHallow | 14 | d47 | WIRED(g6) |
| 172 | RuneWizard | 3 | d47 | WIRED(g6) |
| 173 | Crimera | 2 | d47 | WIRED(g6) |
| 175 | AngryTrapper | 3 | d47 | WIRED(g6) |
| 176 | MossHornet | 2 | d47 | WIRED(g6) |
| 178 | Steampunker | 23 | d47 | WIRED(g6) |
| 179 | CrimsonAxe | 5 | d47 | WIRED(g6) |
| 180 | PigronCrimson | 14 | d47 | WIRED(g6) |

Seam note: mid2's `local_ai0` is INPUT-only (the CorruptPenguin colour bank)
— the g6 bridge feeds it in and does NOT copy it back.

### d48 wave C — `npc_findframe_c` — **WIRED(d44)** (the wire round landed it)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 355 | Firefly | 4 | d48 | WIRED(d44 `9e49ca89`) |
| 356 | Butterfly | 24 | d48 | WIRED(d44 `9e49ca89`) |
| 357 | Worm | 2 | d48 | WIRED(d44 `9e49ca89`) — slope world-read bridge |
| 358 | LightningBug | 4 | d48 | WIRED(d44 `9e49ca89`) |
| 359 | Snail | 6 | d48 | WIRED(d44 `9e49ca89`) |
| 360 | GlowingSnail | 6 | d48 | WIRED(d44 `b3e3f33b`) |
| 361 | Frog | 13 | d48 | WIRED(d44 `b3e3f33b`) |
| 362 | Duck | 15 | d48 | WIRED(d44 `b3e3f33b`) |
| 363 | Duck2 | 15 | d48 | WIRED(d44 `b3e3f33b`) |
| 364 | DuckWhite | 15 | d48 | WIRED(d44 `b3e3f33b`) |
| 365 | DuckWhite2 | 15 | d48 | WIRED(d44 `7c5f0cbd`) |
| 366 | ScorpionBlack | 4 | d48 + d49 twin | g7 high2 hook claims 366 → keep-one fold (§3); the d44 claim helper EXCLUDES it |

g6's dispatch census assigned d48 to g7 ("W4-g7 (second half)"), but g7's
actual commits wired only extra2/extra3/extra4/high2 — `npc_findframe_c` is
referenced NOWHERE in g7's `npc.rs`/`main.rs` (verified by grep at `6ca3e668`).
Net: 11 kinds fully unwired + 366 covered by the g7 scorpion fold. The wire
round must land the `npc_findframe_c` module + a cs_id hook claiming
**355–365 ONLY** (NOT 366 — g7's `find_frame_kind_high2` maps `366 | 367` to
the one Scorpion machine, `NPC.cs:76820-76850` grouped case). Special seam:
the c-module's worm tile-slope probe (`FrameInput::slope`, the caller-resolved
`Framing.GetTileSafely` read, `NPC.cs:72807-72825`) needs world access in the
bridge — the reason g6 kept it out of its world-free lane.
**DONE on this branch (§7)**: the module + hook + `find_frame_worm_slope`
world-read bridge landed; 366 excluded from the claim.

### d49 wave D — `npc_findframe_high2` (hooked only by g7)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 367 | Scorpion | 4 | d49 | WIRED(g7) — one machine for 366+367 |
| 374 | TruffleWorm | 4 | d49 | WIRED(g7) |
| 375 | TruffleWormDigger | 4 | d49 | WIRED(g7) |
| 377 | Grasshopper | 2 | d49 | WIRED(g7) |
| 378 | ChatteringTeethBomb | 4 | d49 | WIRED(g7) |
| 379 | CultistArcherBlue | 12 | d49 | WIRED(g7) |
| 380 | CultistArcherWhite | 12 | d49 | WIRED(g7) |
| 430 | ArmedZombie | 7 | d49 | WIRED(g7) |
| 431 | ArmedZombieEskimo | 7 | d49 | WIRED(g7) |
| 432 | ArmedZombiePincussion | 7 | d49 | WIRED(g7) |
| 433 | ArmedZombieSlimed | 7 | d49 | WIRED(g7) |
| 434 | ArmedZombieSwamp | 7 | d49 | WIRED(g7) |

Seam note: high2 copies FIVE fields (`local_ai[0]` — the truffle worm −2f
pin), feeds `ai[2]` as `ai2` (archer draw / armed-zombie swing ladder).

### e13 extra-1 — `npc_findframe_extra1` (branch-wired; ALL 15 twins)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 121 | Slimer | 4 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 137 | IlluminantBat | 4 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 141 | ToxicSludge | 3 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 142 | SantaClaus | 25 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 148 | Penguin | 12 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 149 | PenguinBlack | 12 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 181 | FaceMonster | 16 | e13 + e14 twin | branch-wired(e13); WIRED(g6+g7 extra2) |
| 182 | FloatyGross | 5 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 185 | SnowFlinx | 12 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 195 | LostGirl | 7 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 196 | Nymph | 14 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 198 | Lihzahrd | 16 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 199 | LihzahrdCrawler | 8 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 201 | HeadacheSkeleton | 15 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |
| 202 | MisassembledSkeleton | 15 | e13 + f26 twin | branch-wired(e13); WIRED(g6+g7 extra3) |

g7's keep-one ruling: **drop `npc_findframe_extra1` entirely** at the merge
(the module is NOT landed on g7; every arm was compared side-by-side). The
one real field deviation: e13's penguin arm reads `FrameState.local_ai0`,
e14's reads `FrameInput.local_ai0` — the g7 bridge feeds BOTH slots and
copies the state slot back, so either feed resolves.

### e14 extra-2 — `npc_findframe_extra2` (hooked by g6 full / g7 trimmed)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 84/93/94/101/102/103/116/120 | (the 8 d45 twins, see d45 table) | — | e14 (+d45) | g6 extra2 claims them; **g7 lookup TRIMMED to exclude** (`86505082`) — they ride the d45 low2 hook. Arms stay in the module as parity references. |
| 121 | Slimer | 4 | e14 (+e13) | WIRED(g6+g7) |
| 137 | IlluminantBat | 4 | e14 (+e13) | WIRED(g6+g7) |
| 141 | ToxicSludge | 3 | e14 (+e13) | WIRED(g6+g7) |
| 142 | SantaClaus | 25 | e14 (+e13) | WIRED(g6+g7) |
| 148 | Penguin | 12 | e14 (+e13) | WIRED(g6+g7) |
| 149 | PenguinBlack | 12 | e14 (+e13) | WIRED(g6+g7) |
| 181 | FaceMonster | 16 | e14 (+e13) | WIRED(g6+g7) |

### f26 extra-3 slice 3 — `npc_findframe_extra3` (hooked by g6 AND g7)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 182 | FloatyGross | 5 | f26 (+e13) | WIRED(g6+g7) |
| 185 | SnowFlinx | 12 | f26 (+e13) | WIRED(g6+g7) |
| 195 | LostGirl | 7 | f26 (+e13) | WIRED(g6+g7) |
| 196 | Nymph | 14 | f26 (+e13) | WIRED(g6+g7) |
| 198 | Lihzahrd | 16 | f26 (+e13) | WIRED(g6+g7) |
| 199 | LihzahrdCrawler | 8 | f26 (+e13) | WIRED(g6+g7) |
| 201 | HeadacheSkeleton | 15 | f26 (+e13) | WIRED(g6+g7) |
| 202 | MisassembledSkeleton | 15 | f26 (+e13) | WIRED(g6+g7) |
| 203 | PantlessSkeleton | 16 | f26 | WIRED(g6+g7) |
| 204 | SpikedJungleSlime | 2 | f26 | WIRED(g6+g7) — `ai_action` seam (num2 motion class) |
| 205 | Moth | 3 | f26 | WIRED(g6+g7) |
| 209 | Cyborg | 26 | f26 | WIRED(g6+g7) |
| 217 | CochinealBeetle | 2 | f26 | WIRED(g6+g7) |
| 218 | CyanBeetle | 2 | f26 | WIRED(g6+g7) |
| 219 | LacBeetle | 2 | f26 | WIRED(g6+g7) |

### f27 extra-4 — `npc_findframe_extra4` (hooked only by g7)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 233 | HornetLeafy | 3 | f27 | WIRED(g7) |
| 234 | HornetSpikey | 3 | f27 | WIRED(g7) |
| 235 | HornetStingy | 3 | f27 | WIRED(g7) |
| 236 | JungleCreeper | 5 | f27 | WIRED(g7) |
| 237 | JungleCreeperWall | 4 | f27 | WIRED(g7) |
| 238 | BlackRecluseWall | 4 | f27 | WIRED(g7) |
| 239 | BloodCrawler | 5 | f27 | WIRED(g7) |
| 240 | BloodCrawlerWall | 4 | f27 | WIRED(g7) |
| 241 | BloodFeeder | 6 | f27 | WIRED(g7) |
| 242 | BloodJelly | 7 | f27 | WIRED(g7) |
| 243 | IceGolem | 15 | f27 | WIRED(g7) |
| 244 | RainbowSlime | 4 | f27 | WIRED(g7) |
| 251 | Eyezor | 3 | f27 | WIRED(g7) |
| 252 | Parrot | 5 | f27 | WIRED(g7) |
| 253 | Reaper | 3 | f27 | WIRED(g7) |

### f26 extra-5 slice 5 — `npc_findframe_extra5` — **WIRED(d44)** (the wire round landed it)

| id | kind | frames | ported-by | wired-by |
| --- | --- | --- | --- | --- |
| 254 | ZombieMushroom | 3 | f26 slice5 | WIRED(d44 `709045f5`) |
| 255 | ZombieMushroomHat | 3 | f26 slice5 | WIRED(d44 `709045f5`) |
| 256 | FungoFish | 4 | f26 slice5 | WIRED(d44 `709045f5`) |
| 257 | AnomuraFungus | 5 | f26 slice5 | WIRED(d44 `709045f5`) |
| 258 | MushiLadybug | 6 | f26 slice5 | WIRED(d44 `709045f5`) |
| 268 | IchorSticker | 4 | f26 slice5 | WIRED(d44 `d050833a`) |
| 269 | RustyArmoredBonesAxe | 15 | f26 slice5 | WIRED(d44 `d050833a`) |
| 270 | RustyArmoredBonesFlail | 15 | f26 slice5 | WIRED(d44 `d050833a`) |
| 271 | RustyArmoredBonesSword | 15 | f26 slice5 | WIRED(d44 `d050833a`) |
| 272 | RustyArmoredBonesSwordNoArmor | 15 | f26 slice5 | WIRED(d44 `d050833a`) |
| 273 | BlueArmoredBones | 15 | f26 slice5 | WIRED(d44 `a6a62eb1`) |
| 274 | BlueArmoredBonesMace | 15 | f26 slice5 | WIRED(d44 `a6a62eb1`) |
| 275 | BlueArmoredBonesNoPants | 16 | f26 slice5 | WIRED(d44 `a6a62eb1`) |
| 276 | BlueArmoredBonesSword | 15 | f26 slice5 | WIRED(d44 `a6a62eb1`) |
| 277 | HellArmoredBones | 15 | f26 slice5 | WIRED(d44 `a6a62eb1`) |

The module already ships its `find_frame_kind_extra5(i16)` lookup — the
wire round only needs the module landing + registration + a cs_id hook +
bridge (the g6 extra3 shape). Note: the task brief's "f27 slice4 15 kinds
not yet in g6/g7 hooks" is STALE — g7 wired all 15 extra-4 ids at `6ca3e668`
(`ba1319a8` first 8 + `63b66a4e` widened to 15). The real second UNWIRED
block is f26 **slice 5** above, not f27 slice 4.
**DONE on this branch (§7)**: module + registration + hook + the minimal
vx/vy/direction bridge landed (no ai/wet per the f26b census).

## 3. The g6-vs-g7 twin-hook union plan (first-match-wins identical)

Both wire lanes branch from main `a725acb0` and prepend cs_id hooks to the
SAME `npc.rs::find_frame` head — the merge is textual-conflicting but
behaviourally trivial:

1. **Hook order after union** (ascending-id, the two lanes' own orders
   compose): low1 (g6) → low2 (d45) → mid1 (d46) → extra2 → mid2 (g6) →
   extra3 → extra4 (g7) → high2 (g7). The sets are disjoint after the
   trims below, so order only matters for the untrimmed twins.
2. **extra2 twin (g6 15-id mapper vs g7 7-id mapper): KEEP g7's.** g6's
   `b58d8d2f` mapper claims 84/93/94/101/102/103/116/120 which the d45
   low2 hook already owns; g7's `86505082` trim removed exactly those 8
   (arms verified verbatim identical, kept in the module as parity
   references). Keeping g6's untrimmed mapper would still be behaviour-
   identical (first-match-wins, twins verbatim) but double-claims ids and
   breaks g7's `w4_g7_the_extra2_lookup_maps_only_the_seven_unclaimed_ids`
   census witness — so keep g7's file version (`6ca3e668`), which also
   feeds the penguin bank through BOTH `FrameInput.local_ai0` and
   `FrameState.local_ai0`.
3. **extra3 twin (g6 vs g7): identical machines.** The g6↔g7 module diff
   is comments-only (9 lines: the dead-code allow note + the mapper doc
   comment); mappers claim the same 15 ids, bridges the same four fields
   + `ai`/`ai_action` inputs. Keep either; g7's is the later doc state.
4. **e13 extra1: DROP the module + hook** (g7's full keep-one dedupe —
   all 15 ids twinned onto extra2/extra3, arm-by-arm verified; the e13
   ANALYSIS/PORT_MAP claims are superseded). If the merger instead keeps
   e13's hook, first-match-wins still yields identical frames — but three
   modules for 15 ids is dead weight; the drop is cleaner.
5. **366 scorpion fold: g7 owns it.** `find_frame_kind_high2` maps
   `366 | 367` → the ONE Scorpion machine (the C# groups the cases). The
   future d48 hook must claim 355–365 only.
6. **Field-seam union** (the bridges differ per module — carry each
   module's own bridge verbatim): low1/low2/mid1 four-field; mid2
   `local_ai0` input-only; extra2 five-field both ways; extra3 four-field
   + `ai_action`; extra4 four-field + `wet`/`ai`/`ai_action`; high2
   five-field + `ai2`; the pending c-module adds the `slope` world-read
   input.
7. **Witness union**: g6's 6 + g7's 12 seam witnesses are per-module and
   hook-order-independent (each pins its own mapper + bridge); the two
   `ALL_KINDS` inertness sweeps compose. g7's `6ca3e668` re-derived three
   module witness pins (SpikedJungleSlime/RainbowSlime counter-0 step tick,
   hornet 8-tick band table) — take g7's extra3/extra2 module versions so
   the corrected pins ride.

## 4. d45/d46/d48 branch-wired status (explicit)

* **d45**: SELF-WIRED on its own pushed branch (`aea0aebd`, +104 lines in
  `npc.rs`: `find_frame_wave_low2` bridge + cs_id hook + seam tests). g6
  left it ("already wired"). No wire-lane action needed beyond the merge.
* **d46**: SELF-WIRED on its own pushed branch (`055fcf8c`, +95 lines,
  same shape). g6 left it. No action needed.
* **d48**: **NOT branch-wired** (`3eb2ee08` touches no `npc.rs`) and NOT
  wired by g6 (assigned away) nor g7 (never landed despite the assignment).
  11 kinds (355–365) need a fresh wire; 366 is already covered by g7's
  scorpion fold. This is the census's biggest find: the g6 dispatch table
  says "W4-g7 (second half)" but g7's own ANALYSIS opening line wrongly
  assumes g6 took d48 — the assignment fell between the two lanes.
  **RESOLVED**: the d44 wire round (§7) landed the module + the
  355–365-only hook + the worm slope world-read bridge on
  `task/W4-d44-findframe-low-1`.

## 5. UNWIRED list — the wire-round worklist

| block | module | ids | count | notes |
| --- | --- | --- | --- | --- |
| ~~d48 wave C~~ | `npc_findframe_c` | 355–365 | 11 | **DONE — WIRED(d44)** on this branch: module verbatim (`e94c7c5b`, blob 71bf90d1 ≡ 3eb2ee08) + cs_id hook claiming 355–365 ONLY (366 excluded, g7 fold) + the worm `slope` world-read bridge (`find_frame_worm_slope` + `find_frame_world` at both production call sites), batches `9e49ca89`/`b3e3f33b`/`7c5f0cbd` |
| ~~f26 slice 5~~ | `npc_findframe_extra5` | 254–258, 268–277 | 15 | **DONE — WIRED(d44)** on this branch: module verbatim (`bd644fd5`, blob 273d6f3e ≡ 73150c56) + cs_id hook + minimal vx/vy/direction three-field bridge (no ai/wet/ai_action/rotation, no world read), batches `709045f5`/`d050833a`/`a6a62eb1` |
| f26c slice 6 | (unpushed) | unknown (278/279/280 head per slice-5 note) | ? | in-flight; re-census after push |
| f27b slice 7 | (unpushed) | unknown | ? | in-flight; re-census after push |
| residual pool | — | the rest of the 219-kind unclaimed pool (204 after slice 5) | ~189+ | port rounds first, then wire |

Already-wired-but-not-on-main reminder: EVERYTHING in §2 lives only on
the pushed task branches — main `a725acb0` has none of the cs_id hooks.
The merger lands g6 ∪ g7 (with §3's keep-one resolutions) + the d45/d46
self-wired branches + THIS branch's two wire blocks (d48 + extra5);
only the in-flight slices remain when they push.

## 7. Wire-round addendum (2026-11-21, this branch)

The two UNWIRED blocks above are now wired on
`task/W4-d44-findframe-low-1` (code-only, no cargo/tests invoked):

* **d48 wave C**: `npc_findframe_c.rs` copied byte-exact from
  `3eb2ee08` (blob `71bf90d1` hash-verified). The hook
  `find_frame_kind_c_claimed` = the module mapper minus ONLY the 366
  scorpion (g7's `find_frame_kind_high2` owns `366 | 367`). The
  five-field bridge `find_frame_wave_c` feeds wet/ai[2]/local_ai[2]
  and copies local_ai[0] back (the worm −2f pin). The worm's slope
  probe is CALLER-RESOLVED per the module contract
  (`NPC.cs:72807-72825`): `find_frame_worm_slope` reads
  `world.tile((int)Center.X/16, (int)position.Y/16)`, falls to the
  tile BELOW on slope 0, and treats out-of-world as slope 0
  (`Framing.GetTileSafely`'s `new Tile()`, `Framing.cs:426-439`).
  `find_frame_world` (the world-read entry) is wired at BOTH
  production `find_frame` call sites — `npc.rs::update_rng`'s tail
  and `world_sim/npcs.rs`'s slot loop; the world-free `find_frame`
  keeps slope 0 for the solo/test paths. Stream-inert: no current
  kind's cs_id lands in 355–365 (hand rows + bestiary net ids swept),
  no arm draws RNG.
* **f26 slice 5**: `npc_findframe_extra5.rs` copied byte-exact from
  `73150c56` (blob `273d6f3e` hash-verified). The hook rides the
  in-module `find_frame_kind_extra5` lookup (all 15 ids). The bridge
  `find_frame_wave_extra5` is MINIMAL: row/counter/sprite_dir both
  ways, vx/vy/facing inputs only — no slice-5 arm reads wet/ai/
  aiAction or writes rotation (the f26b census), so no world read is
  needed and the hook sits in the world-free `find_frame` directly.
  Stream-inert: no current kind's cs_id lands in 254–258/268–277, no
  arm draws RNG.
* **Witnesses**: 7 seam tests in `npc.rs::w4_d44_wire_tests` (both
  inertness sweeps over ALL_KINDS, the 355–365-only claim set with
  the 366 exclusion, the full 15-id extra5 claim set, both bridges'
  field copy-backs with per-machine frame pins, the worm slope
  probe's world reads incl. the below-tile fall-through and
  out-of-world, the world-entry slope resolution) + the modules'
  own 12 + 10 frame witnesses ride verbatim. Written not run.

## 6. Verification performed

Code-only census — no build/test/lint (per lane rules). Evidence commands:
`git ls-remote origin` (all 12 tips match local), `git show
origin/<branch>:crates/tr-app/src/npc_findframe_*.rs` (every kind table row
read from the pushed `vanilla_type`/`frame_count`/mapper arms), `git diff
origin/main...origin/task/W4-g6…/g7… -- crates/tr-app/src/npc.rs` (hook
censuses), `git diff g6:…extra2.rs g7:…extra2.rs` (the 8-id trim) and
`g6:…extra3.rs g7:…extra3.rs` (comments-only, 9 lines), grep for
`findframe_c` on g6/g7 (zero hits — the §4 find). All reads were direct
file paths / git object reads; no junction recursion, ≤100-line windows.
