# 369 — W4-d34 SEAL: the band-211-280 bible + the roster/fighter/islands receipts

> **RENAME CHAIN: 347 (self-pick at `d652e147e`, contested) →
> 369 (registry-150 §9 Lead assignment).** The d54 collision registry
> (`150-research-number-registry.md`, §9, read at
> `origin/task/W4-d54-dust-sites-world` `5869198c5` — "d34-347-self-369
> (g15-347-executed-wins)") renumbers this doc: g15's §9 assignment was
> EXECUTED at `648d2abb7` (`347-item-dust-family-bible.md`), so the
> d34 post-§9 self-pick into 347 is contested → **369**. All refs
> re-pointed this commit (S5 below + the two patch-file headers);
> NO SELF-PICKS — §9 assignments only.

Status: SEAL (doc + witness patches only — zero code diff on this branch).
Author lane: W4-d34 seal executor, on `task/W4-d34-npc-kinds-211-280` cut at
the co-lane tip `f0d92564e` (phase 4). Registry: key **369** per
research/150 §9 (Lead amendment, `5869198c5`). The original seal round
self-picked 347 under the older §7.4 rules (origin
`task/W4-d54-dust-sites-world` `0426245f2`: floor 308+, **no max+1
picks**) after an all-ref first-add sweep that missed g15's executed
§9 assignment `648d2abb7` — the pick was contested and renumbered
347→369 (chain banner above); neighbours 345/346/348/349 are taken
(d40-seal / g65-family / band-85-93-retire / snail-family).

**Co-lane status: ALIVE, NOT clobbered.** At seal start `TR-W4-d34` sat on
`task/W4-d34-npc-kinds-1-70` with 48+ untracked `tools/_d34_*` scripts, fresh
writes mid-session (`_d34_p4probe2.py` 14:27, phase-4 commit `f0d92564e`
pushed 15:17, `_d34_p4spot.py`/`_d34_newdup.py` 15:12-15:13). This seal was
committed via a TEMP index + `commit-tree` + `push HEAD:refs/heads/...` —
the co-lane's checkout, branch ref and untracked files were never touched.
The requested branch name `task/W4-d34-npc-kinds-211-280` did not exist;
band 211-280 is owned/pushed by **d37** (`750d8e446`), so this lane carries
the seal verdicts, not a second port of the band.

Tips read this round (all `git show`/`git grep` at origin refs, direct
paths, no junction recursion):

| Lane | Tip | State |
| --- | --- | --- |
| d34 `npc-kinds-1-70` | `f0d92564e` | phase 4, +20 kinds proto 266-285, ALL_KINDS 290/290 |
| d37 `npc-kinds-211-280` | `7bea5c835` | LAGGARD-EXEC (research/250); code diff `750d8e44..HEAD` EMPTY re-verified |
| d38 `npc-kinds-281-350` | `b888a9575` | owns proto 281-285 (five hardmode casters, `f908a54dd` b1) |
| d43 `npc-kinds-631-697` | `6fbe06250` | retire claims-table (research/244), 63 kinds, ALL_KINDS 333/333 |
| g8 `npc-kind-wire-1` | `dd3cb0572` | E8 EXECUTED (`9c66afc30`): 17 owned → 745-761, Vampire → 762, decl 305/305, 17 dups DROP-PENDING |
| g44 `biome-sky-lakes` | `f688ab142` | SEAL-VERIFY: research/87 STANDS + two supersessions (feather ACTIVE, wall hunk consumed) |
| c4 `style44-flying-fish` | `8abf0f1d8` | SEAL-VERIFY: 5 drops owed incl CorruptGoldfish@405 → d34@277 |
| d54 registry | `0426245f2` | research/150 §7.4, live 148 assignments, 308+ free |

## S1 — git-log census: what landed for band 211-280

`git log origin/task/W4-d37-npc-kinds-211-280` (band owner, 8 commits over
base `a725acb04`):

1. `750d8e446` — THE PORT: NPCID 211-280 coverage, census 65 kinds, 46
   missing ids → **46 new kinds, protocol 354-399 port-local**
   (`npc/data.rs:597-642`), bestiary ENTRIES +43 (Hide-folds 230/236/239:
   GoldfishWalker→230, JungleCreeper→JungleCreeperWall 237,
   BloodCrawler→BloodCrawlerWall 240; cs_id rows ride the vanilla type at
   `npc.rs:19569+`, FlyingFish/FlyingSnake ride 359/360 — the vanilla
   224/226 collide with the Wyvern keys), sheets +46, ALL_KINDS 316.
2. `08f7d25cf` `bdd59afba` `a341daa12` `0466ecf81` — reconcile rounds
   (research/108 playbook → EXTEND 354-ref sweep → RECEIPTS R0-R9+E8 →
   the c9 ALL_KINDS fix-hunk). Doc + witness only.
3. `fbb948172` — ROSTER SEAL S0-S4 (research/108): E8 receipts, count pins
   core 552/489/463/419/103 max 766, merger bible
   d34→d38→d37→d40→d42→d35-renumber-last→small-lanes.
4. `2ea22ce3e` — LAGGARD-VERIFY (108 LV): d125 regression resolved, g8
   ALIVE, c9 fix absorbed locally, lag-union gates PASS.
5. `7bea5c835` — LAGGARD-EXEC (research/250): six roster items closed at
   live tips; FINAL v2 **389 kinds / 389 distinct ids / zero base clash /
   max 823** scripted-pass; per-lane move/drop tables
   `tools/_d37_lag_tables2.md`; **d34 Crab/DungeonGuardian → 791/792**
   (d38 owns 282/283) + PinkJellyfish/Antlion/SpikeBall drop to g8
   745/746/747. Zero code diff re-verified this round.

Neighbouring landings that touch the band's numbers:
- **d34** `30bfb41be`/`4012a7f3a`/`467c9c886`(repair)/`5f3435a87`/`f0d92564e`
  — kinds 1-70 phases 1-4, proto **266-285** (`data.rs:592-611`), the
  281-285 tail now HARD-collides with d38's casters (see S2).
- **d38** `f908a54dd` b1 — proto **281-285** RaggedCaster/OpenCoat,
  Necromancer/Armored, DiabolistRed (`data.rs` rows) — the vanilla-free
  corridor claim that beats d34's phase-4 tail at merge.
- **g8** `9c66afc30`+`dd3cb0572` — E8 move executed; the pre-E8 354-387
  band's overlap with d37's 354-399 is resolved: g8's 17 dups ride
  DROP-PENDING comments (`data.rs:633` Harpy et al), the 17 owned kinds
  ride 745-761. Vampire 388→762 frees d37's RustyArmoredBonesAxe@388.
- **c4** `8abf0f1d8` — FlyingFish adopts d37@359 (`npc.rs:18865`,
  `data.rs:601`), CorruptGoldfish rides 405 pending its drop to d34@277.

**Band verdict:** the 211-280 port itself is COMPLETE and UNCONTESTED at
354-399 (d37 keeps, ZERO-MOVE receipt consumed); every foreign claim on
the corridor (g8's old 354-387, c9's snails 359/360, c4's FlyingFish 354)
has been moved or drop-listed by E8/SEAL/LAGGARD-EXEC. The remaining
band-adjacent collisions are d34's phase-4 tail 281-285 vs d38 — resolved
by d37's tables (2 moves + 3 drops for d34), NOT by re-porting anything.

## S2 — d37e-roster-E8: the d34 keep-final-ids receipt

d34 FINAL ids after the d37 LAGGARD-EXEC tables (`_d37_lag_tables2.md`
§d34, tip `f0d92564e`) — **15 keeps / 2 moves / 3 drops**:

| Kind | d34 pushed | FINAL | verdict |
| --- | --- | --- | --- |
| AngryBones | 266 | **266** | KEEP |
| CursedSkull | 267 | **267** | KEEP (g8@375 drops) |
| OldMan | 268 | **268** | KEEP |
| ManEater | 269 | **269** | KEEP |
| Bunny | 270 | **270** | KEEP |
| BoneSerpentHead/Body/Tail | 271/272/273 | **271/272/273** | KEEP |
| CorruptBunny | 274 | **274** | KEEP |
| **Harpy** | **275** | **275** | **KEEP — g8@386 drops (S4)** |
| Snatcher | 276 | **276** | KEEP |
| **CorruptGoldfish** | **277** | **277** | **KEEP — c4@405 retires** |
| Vulture | 278 | **278** | KEEP (g8@354 drops) |
| Demon | 279 | **279** | KEEP (g8@376 drops) |
| BlueJellyfish | 280 | **280** | KEEP (g8@355 drops) |
| PinkJellyfish | 281 | g8 **745** | DROP d34 row (g8 owns cs 64) |
| Crab | 282 | **791** | MOVE (d38 owns 282) |
| DungeonGuardian | 283 | **792** | MOVE (d38 owns 283) |
| Antlion | 284 | g8 **746** | DROP d34 row (g8 owns cs 69) |
| SpikeBall | 285 | g8 **747** | DROP d34 row (g8 owns cs 70) |

**CorruptGoldfish keep-d34@277 / retire-c4@405** — exact drop sites at c4
tip `8abf0f1d8` (from d37's table + this round's re-read):
- `crates/tr-app/src/npc/data.rs:621` `NpcKind::CorruptGoldfish => 405,` (to_u16)
- `crates/tr-app/src/npc/data.rs:974` `405 => NpcKind::CorruptGoldfish,` (from_u16)
- `crates/tr-app/src/npc.rs:18875` `NpcKind::CorruptGoldfish => 405,` —
  the cs_id/hash arm: d34's cs_id 57 row survives; c4's comment block
  `:18869-18874` folds its 405 mention. The kind ROW + stat rows stay
  with d34's implementation (style-16 machine `5f3435a87`); c4 keeps its
  world-sim wiring per the FlyingFish precedent.
- d34 survivor rows: `data.rs:603` (277 to), `:953` (277 from),
  cs_id 57 at `data.rs:3245`. Bijection holds on both sides after the drop.

**Harpy keep-d34@275 × g8@386 — ONE loud dup, keep ONE row.** Both lanes
add `NpcKind::Harpy` (enum variant + to/from arms): a duplicate variant
will not compile — the conflict is loud, never silent. Keep-sites:
- d34 `f0d92564e`: `data.rs:601` (275 to), `:951` (275 from), enum +
  stats `npc.rs:19382`, identity witness `(Harpy, 48, 14, 24.0, 34.0,
  25, 8, 100, 0.6, 300.0)` at `npc.rs:24881` — net_id 48, ai_style 14.
- g8 `dd3cb0572`: `data.rs:633` (386 to, DROP-PENDING comment),
  `:1011` (386 from), `npc.rs:20316` (cs-arm), `:25739` (ALL_KINDS row),
  witnesses `:36821/:36842/:37107`. The identity tuple is IDENTICAL
  (`npc.rs:36821`) — either survivor keeps every stat witness green;
  the drop deletes g8's variant+arms+ALL_KINDS row and re-points its
  three witness pins 386→275 (or drops the dup rows wholesale and keeps
  d34's witness). g8's **machine work ALL STAYS** (S4): style-14 axis
  routing `npc.rs:7154` (`Harpy | RedDevil => unreachable!` — the axis
  claims style 14 first), the flap arm `:10660`, the feather machine.

## S3 — d22d-fighter-seal: the fighter_walker_kind extension (d34 × d43)

The S1-D-3z walker gate `ambient_sound.rs:603` `fighter_walker_kind`
(blob **`546b0b9e4`**, BYTE-IDENTICAL at d34 `f0d92564e` AND d43
`6fbe06250`) gates the two live consumers in `world_sim/npcs.rs`
(d34 tip `:4423` idle counter, `:4584` not-discouraged/moan). The d34
kinds wave and the d43 kinds wave each landed NEW plain aiStyle-3
walkers that the gate does not name — their `ai[3]` idle counter and
moan arms are dead until this extension:

| Kind | lane | dispatch evidence (tip) | cs_id | walk cap |
| --- | --- | --- | --- | --- |
| **AngryBones** | d34 | `npc.rs:6497` → `update_skeleton` → `walker_step_fighter` | 31 (`data.rs:3232`) | 1.5 (`data.rs:2620`, num83 `:57606`) |
| CorruptBunny | d34 | `update_skeleton` arm | 47 (`data.rs:3242`) | 1.5 (`:2621`) |
| Crab | d34 | `update_skeleton` arm | 67 (`data.rs:3250`) | 0.5 (`:2712`) |
| MossZombie | d43 | `npc.rs:6752` → `update_zombie` | 691 (`data.rs:3504`) | 0.85 (`:2934`) |
| RockGolem | d43 | `npc.rs:6839` → `update_skeleton` | 631 (`:3557`) | 0.9 (`:2935`) |
| MaggotZombie | d43 | `npc.rs:6840` → `update_zombie` | 632 (`:3558`) | 0.8 (`:2936`) |
| SporeSkeleton | d43 | `npc.rs:6842` → `update_skeleton` | 635 (`:3560`) | 1.5 (`:2937`) |

All seven are `ai_style 3` (d43 rows `data.rs:2768/2821/2822/2824`), all
seven have `ai3_walk_speed` rows (the `walker_step_fighter` `.expect`
already resolves — only the ambient gate misses them). Zero hits for any
of the seven in `ambient_sound.rs` at either tip (git grep exit 1 both).

Moan shapes after the extension (the C# `NPC.cs:57340` list already in
the port at `ambient_sound.rs:888-925`, keyed by **cs_id**): **31
AngryBones, 691 MossZombie, 632 MaggotZombie, 635 SporeSkeleton ARE in
the family 1000-roll list** (the `| 31` `:896`, `3 | 691` `:890`,
`| 632` `:923`, `| 635` `:924` arms) → each gains the case-14
`Next(1000)` draw per qualifying tick (the witnesses pin hit + miss);
**47 CorruptBunny / 67 Crab / 631 RockGolem are in NO moan list**
(`:57340` family, `:57344` 489/586, `:57348` mummies, `:57352` vampire,
`:57356` Frankenstein, `:57360` face monster, `:57364` armored 269-280)
→ counter-only walkers, draw-free (the d37 precedent block). NOTE the
631/632/635/691 keys were ALREADY in the moan list (the base S1-D-3z
port carried the vanilla types ahead of the kinds) — the d43 kinds'
cs_id rows make those arms REACHABLE; no moan-code change is needed,
only the gate extension.
The d37 band already extended the gate for ITS walkers at its own tip
(`ambient_sound.rs:652-680`, blob `571b35246`, +24 kinds incl the armored
269-280 moan family) — that hunk and this one are disjoint insertions at
the same seam (before the closing `)` of the `matches!`); both keep.

Exact published hunks: TWO per-lane patches in this commit (a single
combined patch cannot compile on either lane — d34 lacks the four d43
variants and d43 lacks AngryBones/CorruptBunny/Crab/Harpy). Both are
**zero-context pure-insertion hunks** (`git apply --unidiff-zero`) at
the same two seams of the SHARED pristine blob `546b0b9e4`, so they
compose in EITHER order at the merge with zero textual conflict:
1. `tools/_d34_seal_fighter_gate_d34.patch` — apply on d34 with
   `git apply --unidiff-zero`: after `:651` `| Splinterling` +11 lines
   (the citation comment + AngryBones/CorruptBunny/Crab arms); after
   `:1880` (the tests-module last test's close) +49 lines: witness
   `w4_d34_seal_the_kinds_wave_walkers_are_gated` — the three gated,
   Harpy/Demon (flyers) NOT gated, the AngryBones-31 moan one-draw pin
   (seed 146 hit → `[(14,1)]`, seed 1 miss → empty, `rng.peek()`
   equality — the `s1_d_3z` witness shape) + the 47/67 zero-draw pins.
2. `tools/_d34_seal_fighter_gate_d43.patch` — apply on d43 the same
   way: +14 lines at the gate seam (comment +
   MossZombie/RockGolem/MaggotZombie/SporeSkeleton); +54 lines at the
   tests seam: witness `w4_d43_seal_the_kinds_wave_walkers_are_gated`
   — the four gated, Drippler NOT, the 691/632/635 one-draw pins
   (family list) and the 631 zero-draw pin (no moan arm).
Both witnesses WRITTEN NOT RUN (code-only lane). Inserted text is
ASCII-only (the repo's existing UTF-8 bytes untouched). Verified
mechanically against the byte-exact `546b0b9e4` blob in a scratch repo:
`git apply --check --unidiff-zero` each alone = exit 0; sequential
d34→d43 = both exit 0, result blob `f8a6a9d79`; sequential d43→d34 =
both exit 0, result blob `4ff66739c` (order-independent up to block
order; both carry all 7 arms + both witnesses, brace-balanced). The
d37 band's own gate block (`571b35246`, +24 kinds) inserts at the same
seam and composes the same way — three disjoint insertions, one gate.

Companion receipt (no hunk needed): `npc_ai_fighter_scan.rs` blob
`0a2a15f81` also identical on both lanes; `is_fighter_zombie` (`:406`)
is the ZOMBIE-family drain gate — MossZombie/MaggotZombie belong on it
when d43's scan-drain wiring lands (d43's research/244 activation
contract: no auto-activation, the axis arms + env adapters are the
merge-wave wiring). AngryBones/CorruptBunny/Crab/RockGolem/SporeSkeleton
are skeleton-walkers, NOT zombie-family — they must NOT join
`is_fighter_zombie`. The d37 tables' AngryBones row: no move, no drop —
proto 266 stands (the 294-296 AngryBonesBig trio is d38's, already in
the `:57340` list as vanilla types).

## S4 — g44e-islands: the harpy kind-gate resolves at merge

research/87 RETIRE §6 (`9ce512b4f`, re-verified `f688ab142`): **the
kind-gate resolves CLEANLY at merge** — g44's sky dispatch
(`spawner.rs:798-837/:938-1011`) touches neither g8's nor d34's files
(both spawner diffs vs base EMPTY, re-verified at `f688ab142`); the
semantic join is ONE published activation hunk:

- Replace the terminal-default block `spawner.rs:1003-1010` (8-line
  comment + `return None;`) with the Harpy spawn
  `return Npc::spawn_hover(ctx.world, x, y / 16, NpcKind::Harpy);`
  (the exact hunk text is in research/87 §6, unchanged at `f688ab142`).
- Witness flips (same merge): `the_sky_default_never_falls_through_to_
  the_ground_ladders` `:3586` — the sweep assert at **`:3601`** flips
  `n.is_none()` → `n.is_none_or(|n| n.kind == NpcKind::Harpy)`; the
  wyvern-gates witness `:3527` assert at **`:3554`** flips the same way
  (a town-slime `Next(25)` HIT now spawns the Harpy default). The
  STREAM-equality asserts (`:3553` et al) are untouched — load-bearing.
- **g8-386 × d34-275: keep ONE row = d34@275** (S2; g8's own
  DROP-PENDING comment at `data.rs:633` already names this verdict).
  Either survivor activates the band identically: net_id 48 +
  ai_style 14 on both, and the g44 witnesses reference NO protocol id.
- **Supersession consumed** (`f688ab142`): the §6 "feather shot NOT
  landed anywhere" sweep is STALE — g8 machine round 1 `a6c4f5ae9`
  PORTED it (30/60/90 cadence, 6f, X-then-Y `Next(-100,101)` jitter,
  `400+Next(400)` reset, wet arm, HARPY_FEATHER 38 row + timeLeft-300;
  witness `npc.rs:36850-36880` at g8 tip). A merged Harpy flies,
  approaches, bounces, wets AND shoots. The machine keeps g8's axis
  routing; d34's direct `update_cave_bat` arm (`npc.rs:6507`) dedupes
  onto the axis (same machine either way — 87 §6's table).

## S5 — research FINAL: the registry number + the band bible

**Registry number: 369** (this doc — chain 347-self-contested → 369-§9,
banner above). The original seal round's sweep under the then-live
research/150 §7.4 (`0426245f2`: assignments live 151-236 + 246-280 +
281-303 + 304-307, winners 237-245, floor 308+, no max+1 picks) found
347 free (post-floor claimants 308×4 / 309 / 312 / 341 / 342 / 343 /
345 / 346 / 348 / 349; 308 and 304/305 were new multi-way collisions
for the next d54 refresh — NOT this lane's to adjudicate) and self-picked
it — but g15's §9 assignment was already EXECUTED at `648d2abb7`
(347-item-dust-family-bible), so the self-pick was contested and the
d54 §9 registry (`5869198c5`, "d34-347-self-369") assigned **369**.
Renamed here; §9 floor is now 370+.

**The band-211-280 bible (one paragraph):** d37's port `750d8e446` owns
NPCID 211-280 as protocol **354-399 port-local** (46 kinds, ENTRIES +43
with the 230/236/239 Hide-folds, sheets +46, ALL_KINDS 316, bijection
scripted at every seal round) and KEEPS every id — the ZERO-MOVE receipt
was consumed at `fbb948172` and re-verified at `7bea5c835` (empty code
diff `750d8e44..HEAD`); every corridor squatter resolved elsewhere: g8's
old 354-387 band split into 17 owned moves → 745-761 + Vampire 762
(EXECUTED `9c66afc30`, decl fixed 270→305) + 17 DROP-PENDING dups whose
keepers are d34 (Vulture 278, BlueJellyfish 280, CursedSkull 267, Demon
279, **Harpy 275**), d35 (renumbered 698-730 corridor), d37 itself
(FlyingSnake 360, Squid 358); c9's snails moved 740/741; c4's FlyingFish
adopted 359 and its CorruptGoldfish@405 retires onto **d34@277**; the
armored-bones 269-280 moan family and the 24 counter-only walkers ride
d37's own `fighter_walker_kind` extension, which composes with this
seal's d34/d43 extension (S3) at the same seam; d34's phase-4 tail
281-285 yields to d38's casters (Crab→791, DungeonGuardian→792,
PinkJellyfish/Antlion/SpikeBall drop to g8 745-747); merge order stands
**d34→d38→d37→d40→d42→d35-renumber-last→small-lanes** with the bijection
gate and lazy-load mandatory (108 S4), grand census at LAGGARD-EXEC
389 kinds / max 823 / zero base clash; the harpy sky band activates via
g44's ONE spawner hunk (S4) with the feather machine already landed —
ONE owner, ONE harpy row, every join a published hunk.

## Verification performed (code-only lane)

- `git status` co-lane check ×3 (48→56 untracked, fresh mtimes, phase-4
  push mid-session) → ALIVE, temp-index seal path chosen.
- All row/line receipts above: `git grep -n` / `git show <ref>:<path>`
  at the pinned origin tips (no working-tree reads of foreign lanes).
- `git diff --stat 750d8e446 origin/task/W4-d37-npc-kinds-211-280 --
  crates anchors.txt` → EMPTY (d37 zero-move re-verified).
- Blob identity: `git rev-parse <d34|d43>:.../ambient_sound.rs` → both
  `546b0b9e4`; `.../npc_ai_fighter_scan.rs` → both `0a2a15f81`.
- d34 bijection census (scripted, git-show blobs): to_u16 290 arms /
  from_u16 290 arms / 290 distinct / ALL_KINDS decl 290 rows 290.
- g8 post-E8 array census: 305 rows / 305 unique (decl 305 — the
  `9c66afc30` fix consumed).
- Patch byte-exactness: `git apply --check --unidiff-zero` against the
  byte-exact `546b0b9e4` blob in a scratch repo — d34 alone exit 0,
  d43 alone exit 0; apply sequences d34→d43 and d43→d34 both exit 0
  (results `f8a6a9d79` / `4ff66739c`, all 7 gate arms + both witnesses
  present, braces balanced 229/229).
- Registry sweep (original seal round, chain history): `git log --all
  --diff-filter=A --name-only -- docs/research/*` bucketed
  100-199/200-299/300+ → 347 zero claimants — that sweep missed g15's
  executed §9 assignment `648d2abb7`; the self-pick was contested and
  renamed 347→369 per registry §9 (`5869198c5`).
- No cargo / tests / rustfmt / graphify (lane constraint). The new
  witness is WRITTEN NOT RUN (the code-only convention).
