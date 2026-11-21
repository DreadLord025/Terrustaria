# 125 — PROJ-DUST master coverage census (W4-e3, coverage phase, 2026-11-21)

Status: CENSUS + one window claim. Code-only mode — no cargo/rustc/tests/
rustfmt/graphify invoked. Method: a fresh `Dust.NewDust` line scan of
`decompiled/Terraria/Projectile.cs` (**1195 sites**, first `:16851`, last
`:78599`), each site attributed to its enclosing method (the 341-method
signature scan), then matched against every pushed dust-lane tip's C#
citations (module bodies AND commit messages — the branch list is the claim
record) plus main's own `Projectile.cs` citations (the draw-census stubs).
A site is COVERED when a lane or main cites its line or an enclosing range;
UNCITED otherwise. Citation-range matching is deliberately generous (a
range cite claims its whole span), so the uncited list below is a
conservative floor — nothing in it is claimed by any pushed tip.

## The pushed tips censused (2026-11-21, second fetch — the lanes moved mid-census)

| lane | tip | claim axis | Projectile.cs dust span |
| ---- | --- | ---------- | ----------------------- |
| W4-d50 `dust-type-table` | `2e243a2d` | the pool contract (`Dust.cs` physics chain) | no sites (foundation) |
| W4-d51 `dust-sites-player` | `0d0cf7c4` | Player.cs sites | — |
| W4-d52 `dust-sites-npc` | `231b3155` | NPC.cs sites | — |
| W4-d53 `dust-sites-projectile` | `6d09fe61` | SITE batch 1 (file line order) | `:16851-22297` (sites 1-40) + the wet machine `:16786-17007` (batch 4 `fa7b331d`) + the 466/190/191/728 re-census (batch 3) |
| W4-d54 `dust-sites-world` | `c89aea9f` | WorldGen/Main world sites | — |
| W4-e1 `dust-sites-player-2` | `3245d624` | Player.cs sites 2 | — |
| W4-e2 `dust-sites-npc-2` | `6e304b96` | NPC.cs sites 2 | — |
| **W4-e3 (this branch)** | `8023aa4d` + this round | SITE batch 2 | `:22297-24328` (42 sites, DONE) + **`:78227-78508` (15 sites, THIS round's claim)** |
| W4-e4 `dust-sites-world-2` | `2a79551a` | world sites 2 | — |
| W4-f3/f4/f6 | `b9575da6`/`0b7b14e2`/`0f9fd642` | player/npc/world 3 | — |
| W4-f5 `dust-proj-3` | `2d3829f3` | SITE batch 3, then NPC.HitEffect wave D | `:24406-26501` (43 sites, batches 1-5); wave D is NPC.cs `:85560-89835` (NOT Projectile.cs) |
| W4-f10 `gore-sites-3` | `b5d2cba1` | NPC.HitEffect gore wave C | NPC.cs `:82206-85559` — the dust bursts inside those arms ride f10's strike-stream census, NOT Projectile.cs sites |
| W4-g16 `gore-projectile-sites` | `f1c60ffe` | Projectile.cs GORE census (165 sites, waves A-O COMPLETE) | gore only — every `Kill_Bombs` dust loop is explicitly "the dust lane's" in its module headers |
| W4-g63 `proj-trail-dust-1` | `ed8d4bf5` | per-kind TRAIL, kinds ≤400 | AI() rowed inline kinds (batches 1-4) + AI_001 slice kinds (batch 5) + 17 unrowed emitters UNWIRED (batch 6); doc 66 |
| W4-g64 `proj-trail-dust-2` | `77616a7d` | per-kind TRAIL, kinds ≥401 | 41 kinds / 5 batches + the wiring wave (16 call sites re-armed); doc 66 |
| W4-g65 `proj-death-dust` | `7f0dc864` | Kill() death dust | `:70642-77262` ONLY (module header pins the span); 10 batches ≈147 sites; the i17/i18 special-arrow/bullet deaths stay main's |
| W4-d75/76/77 `proj-sounds` | `0e700d9a`/`ede24bb4`/`d61e6362` | SOUND sites 1-120 / 121-240 / 241-437 | `:16851-41968` / `:41978-70958` / `:71112-78599` — sounds only; the dust siblings are cited skips |
| W4-f48 `dodge-2` | `69d499c6` | the cloak-five rows | 723/724/725/726/1133 ROWED (`:7646-7658` shared row + `ai_cloak_star`) |
| W4-d16 / W4-d1 | `e122a6b2` / `e271b5bc` | the 152 machine / the aiStyle-6 powder | count-only dust (the e3 keep-one contracts, ANALYSIS "the W4-e3 window census") |

## Region coverage (1195 sites, the method-attribution cut)

| region | sites | lane-cited | main-only (census stubs) | UNCITED |
| ------ | ----- | ---------- | ------------------------ | ------- |
| `Update` wet/arrow/enchantment + `EmitEnchantmentVisualsAt` (`:16668-17468`) | 28 | 28 (d53) | 0 | **0** |
| `HandleMovement` + pre-AI named machines (`:17469-23198`) | 15 | 15 (d53/e3) | 0 | **0** |
| `AI()` inline style machines (`:23199-36808`) | 233 | 153 (e3/f5/g63/g64) | 40 | **40** |
| named `AI_xxx` machines (`:36809-54001`) | 251 | 0 | 164 | **87** |
| `AI_001` (`:54002-57825`) | 121 | 15 (g63 batch 5) | 104 | **2** |
| `AI_010/026/062/075/099/130-145` named (`:57826-70641`) | 93 | 0 | 57 | **36** |
| `Kill()` (`:70642-77262`) | 399 | 147 (g65) | 18 | **234** |
| Kill tail: `Kill_ExplodeTiles`/`Kill_Bombs_DoUsualKillCode`/`DoRainbowCrystalStaffExplosion` (`:77265-78614`) | 55 | 0 | 40 | **15** |

**The early-AI region `:0-22297` is FULLY claimed** — d53's sites 1-40 cover
every one of its 43 sites (the wet 8 now WIRED by d53 batch 4, the 466/728
re-censused by batch 3, the 977 four STAGED on e3). Nothing unclaimed there.

**The mid region `:24318-82206`** (the task's ":24318-82206" framing mixes
files — `:82206` is f10's NPC.cs boundary; in Projectile.cs the mid region is
`:24328-70641`): f5 owns `:24406-26501`; g63/g64 own the per-kind trail arms
by KIND (their cites land where the rowed kinds' arms sit); the rest is the
main draw-census stubs + the uncited remainders below.

## UNCLAIMED windows (the uncited-site list, by host)

1. **`Kill()` remainder — 234 sites** (`:70753-77204`, scattered). g65's 10
   batches took ~147; the rest are the kinds g65 deferred (the gore-interleaved
   arms its module header defers, the unrowed kinds, the `:70753-71103` head
   block — Digtoise 1098/1123, the Seraph necklace, the 932 census twin — and
   the `:76689-77055` tail). FIRST uncited: 70753/70869/70875/70878/70884/
   70909/70939/70963/70983/71011/71024/71037; LAST: 77036/77040/77051/77055/
   77166/77182/77200/77204. **Owner: g65's continuation** (its claim is the
   whole `Kill()` span; the uncited sites are its documented remainder, not a
   free window).
2. **`Kill_Bombs_DoUsualKillCode` dust tail — 15 sites, `:78227-78508`**:
   78227/78240/78250/78254/78269 (773 scarab), 78340/78345/78348 (the
   28/30/37/75/102/164/397/517/516/519 bomb family), 78407/78412/78415 (the
   29/108/470/637/1002/1086/1087 big bomb), 78457/78464/78467 (588 confetti),
   78508 (1088 acorn). g65's span ENDS at `:77262`; g16 owns only the gore
   sites there (waves M/O explicitly leave "the dust loops before each" to
   the dust lane); main's `rocket_kill_census` covers `:77648-78210` only.
   **THE SMALLEST genuinely unclaimed contiguous window — CLAIMED BY W4-e3
   THIS ROUND** (the port below).
3. **`AI()` inline remainder — 40 sites**: 27869, 28833, 29245/29284, 31017/
   31053/31149/31207, 31896/31928, 32305/32330/32426/32548, 32958/33015/33147,
   33993/34005/34020/34123, 34507-34588 (8), 35601-35986 (7), 36106/36157/
   36170, 36427. These are the UNROWED-kind arms g63/g64's batch-6 convention
   covers by emitter-but-unwired where the kind was censused; the lines above
   have NO emitter on any tip (the style-30 gem sparkle tail, the 60 last-prism
   remainder, the style-68/66 hostile twins, the aiStyle-132/135/137 blocks).
   **Owner: the g63/g64 continuation** (per-kind axis).
4. **Named-machine region — 87 + 36 sites** (`:36809-54001` / `:57826-70641`):
   the biggest blocks are AI_192_JuminoAnimation (8, `:41525-41627`),
   AI_190_NightsEdge (11, `:42034-42216`), AI_075 (11), AI_019_Spears_Old (8,
   `:46490-46530`), AI_015_Flails_Dust (7, `:44410-44470`), AI_178_FireExplosion
   (5), AI_140_MonkStaffT1 (5), AI_009_MagicMissiles_Old (5, the 491 unrowed),
   AI_205_RemoteControlCar (4), AI_203_StormLightning (4), AI_191_TrueNightsEdge
   (4), plus the 1-3-site machines (AI_208/053/029/040/204/202/200/199/197/195/
   194/193/186/184/181/179/177/171/169/168/162/161/157/155/153/150/147/145/144/
   143/142/137/136/135/131/130/123/121/120/113/111/099/061/067/062/026/010/003/
   009). Main's machines carry draw-census stubs for 164+57 of these; the
   uncited rest are the unported-machine dust (the Jumino/StormLightning/
   RemoteControlCar/Digtoise/FirstFractal/Smolstars/ToiletPop/RapierStabs/
   IceWhipSlicer/FireExplosion/BookStaffStorm machines have NO port at all).
   **Owner: unassigned — the named-machine dust wave** (a future lane per
   machine family; NOT the site-batch axis, which stopped at f5's `:26501`).
5. **`AI_001` — 2 sites**: 56049, 56748 (the g63 batch-5/6 remainder edges).
   **Owner: g63's continuation.**

## THIS ROUND'S PORT — the `:78227-78508` window (15 sites)

The e3 emitter pattern (`ProjDust` bundle, argument draws FIRST, pool
internals, post-writes on the returned slot incl. the sentinel; gore TYPE
argument draws kept on the stream, bodies g16's — the `style2_burst_31_6`
precedent). Two commits, ≤10 sites each:

- **Wired (6 sites)**: the bomb-family arm `:78340/78345/78348` (the
  `explode_bomb` kinds 28/37/516/519 + the grenades 30/397/517 — the C#
  `:78329` list; 75/102/164 cited: 75 unrowed, 102/164 are the blast rows the
  port's `explode_bomb` kind gate refuses) and the big-bomb arm
  `:78407/78412/78415` (the dynamite 29; the 108/470/637/1002/1086/1087 halves
  cited — 108 rowed but refused by the same gate, the rest unrowed). The
  `explode_bomb` seam gains `rng`+`DustPool`+`SpawnCtx` (the `explode_rocket`
  precedent); the frozen zero-rng M4-PROJ-9 path's DUST half is re-armed, the
  `damage_pve` FastRandom(0) stand-in stays (its own documented deviation).
- **Staged unwired (9 sites)**: the 773 scarab five (`:78227-78269` — unrowed
  + the `GetScarabBombDigDirectionSnap8` player seam), the 588 confetti three
  (`:78457-78467` — unrowed), the 1088 acorn one (`:78508` — rowed, but the
  port's generic `kill()` does not run the aiStyle-16 `Kill_Bombs` tail for
  non-detonated deaths; the host gap is cited). Emitters + draw-mirror
  witnesses land now (the e3 `star_slash_dust` STAGED precedent); the wiring
  ride the rows/host wave.

Keep-one at merge: g16 wave O's `bomb_corner_gores`/`big_bomb_gores`/
`confetti_bomb_gores`/`scarab_bomb_gores` (unwired, `projectile_gore_o.rs`)
replace the TYPE-only draws kept here — the type draw moves into the
`new_gore` argument position, the same stream slot; the kicks
(`Next(10,31)`/`Next(-20,21)`) are the gore lane's internals and join there.

## The 503/1037/1049 UNROWED-kind census + the 3-row port (this branch)

C# `SetDefaults` rows (the id-table region `:697-10920`):

- **503 StarWrath** `:5743-5756`: 24×24, aiStyle 5, friendly, penetrate 2,
  alpha 255, tileCollide false, **melee**, extraUpdates 1,
  usesLocalNPCImmunity + localNPCHitCooldown 15.
- **1037 MeteorWhipMeteor** `:9780-9792`: 24×24, aiStyle 5, friendly,
  usesLocalNPCImmunity + localNPCHitCooldown −1, scale 0.8, extraUpdates 1,
  alpha 254, tileCollide false (penetrate keeps the stock 1, timeLeft 3600).
- **1049 MeteorStormMeteor** `:9938-9952`: 24×24, aiStyle 5, friendly AND
  hostile, scale 0.8, timeLeft 1200, extraUpdates 1, alpha 254, penetrate −1,
  tileCollide false, trap. (The hostile half rides the port's `spawn_hostile`
  friendly=false flip — the FLAMES_TRAP precedent; the `netMode == 2`
  localAI[0] counter `:23832-23840` is server-only, dropped.)

All three are ported VERBATIM on this branch (`projectile/data.rs`), which
unblocks the e3 `ai_falling_star_family` kind gate (the arm + the
`style5_spark_503`/`style5_comet_tail` emitters were already written, batch 3)
and g64's unwired `trail_503` twin. Cross-notes:

- **f48 keep-one** (the cloak five 723-726/1133): unchanged — f48's rowed
  `ai_cloak_star` survives the merge and takes the cloak dust half; the
  503/1037/1049 halves keep the e3 arm (ANALYSIS "the W4-e3 window census").
- **g64 keep-one**: `trail_503` (unwired at g64's tip, now wireable via this
  row) duplicates `style5_spark_503` draw-for-draw — keep ONE at the wiring
  wave; the e3 arm is the rowed survivor on this branch.
- **d75 sound owed**: with 503 rowed, the `:23881-23885` hum (`20 + Next(40)`,
  the `type != 1037 && type != 1049` gate) now applies to the 503 too — d75's
  re-arm covers the 9/955 only; the 503 extension is the sound lane's merge
  edit. The e3 arm keeps the hum skip cited until then (the batch-3
  convention).
