# 125 — PROJ-DUST master coverage census (W4-e3, coverage phase, 2026-11-21; REFRESHED same day, the AI-INLINE-40 round)

Status: CENSUS + two window claims (the `:78227-78508` Kill_Bombs tail,
DONE; the AI-INLINE-40 + AI_001-edges-2 window, DONE this refresh — see the
round-2 sections at the bottom). Code-only mode — no cargo/rustc/tests/
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
| **W4-e3 (this branch)** | `c9206e75` + this refresh | SITE batch 2 + the two coverage windows | `:22297-24328` (42 sites, DONE) + `:78227-78508` (15 sites, DONE) + **the AI-INLINE window 37 sites + the AI_001 edges 2 (THIS refresh, below)** |
| W4-e4 `dust-sites-world-2` | `2a79551a` | world sites 2 | — |
| W4-f3/f4/f6 | `b9575da6`/`0b7b14e2`/`0f9fd642` | player/npc/world 3 | — |
| W4-f5 `dust-proj-3` | `55dd8f9f` (refresh) | SITE batch 3, then NPC.HitEffect wave D (batches ≤96) | `:24406-26501` (43 sites, batches 1-5); wave D is NPC.cs `:85560-89835` (NOT Projectile.cs) |
| W4-f10 `gore-sites-3` | `b5d2cba1` | NPC.HitEffect gore wave C | NPC.cs `:82206-85559` — the dust bursts inside those arms ride f10's strike-stream census, NOT Projectile.cs sites |
| W4-g16 `gore-projectile-sites` | `f1c60ffe` | Projectile.cs GORE census (165 sites, waves A-O COMPLETE) | gore only — every `Kill_Bombs` dust loop is explicitly "the dust lane's" in its module headers |
| W4-g63 `proj-trail-dust-1` | `2ea3e711` (refresh) | per-kind TRAIL, kinds ≤400 | AI() rowed inline kinds (batches 1-4) + AI_001 slice kinds (batch 5) + 17 unrowed emitters UNWIRED (batch 6) + the 6 owed AI_001 emitters (batch 7: 55/174/176/299/374/376) + research/67 activation checklist; doc 66 |
| W4-g64 `proj-trail-dust-2` | `625c449e` (refresh) | per-kind TRAIL, kinds ≥401 | 41 kinds / 5 batches + the wiring wave + remainders 1-4: the style-124 machine dust-wired (**claims `:35231/35295/35601/35681`**), the 813 four-phase bodies (**claims `:34020`** + the `:33951-33983` dissolve), the 596 row, the e3/g63 keep-one refresh; doc 66 |
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
   36170, 36427. **RESOLVED THIS REFRESH**: the re-probe against the moved
   tips found 3 now g64-claimed (`:35601/:35681` remainder 1's
   `trail_650_scan_sparkle`/`trail_650_point_sparkle`; `:34020` remainder
   2's `trail_813_opacity_tail` — the `:34018-34024` range cite); the other
   **37 are PORTED by W4-e3** (34 staged + 3 wired — the round-2 ledger
   below). The g63 doc-66 census row "60 | 358 (`:29167/.../29245/29284`)"
   MIS-ATTRIBUTES the last two: `:29234`'s `type != 1042` return precedes
   them — they are the **1042 ShimmerGunStream's** (the e3
   `style60_shimmer_gun_trail_1042` correction; g63's 358 emitter covers
   `:29167/29191` only, its `:29158-29196` range cite stops before them).
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
   **Owner: ASSIGNED THIS REFRESH — the per-machine assignment table at the
   bottom of this doc** (which d-lane owns which machine → the
   dust-activation contract per merge; the round-1 "unassigned" status is
   superseded).
5. **`AI_001` — 2 sites**: 56049, 56748 (the g63 batch-5/6 remainder edges).
   **RESOLVED THIS REFRESH — both ported by W4-e3**: the 56049 is the 1006
   ShimmerArrow's rainbow sparkle + CloneDust twin (WIRED — the 1006 row +
   `special_arrows` arm live on this base); the 56748 is the 981
   SilverBullet's 1-in-6 dust-279 (STAGED — no row on any pushed tip; the
   g63 batch-7 remainder census did not take these two: its 20-kind ≤400
   list excludes the 981/1006 >400 edges, and g64's 41-kind ledger has
   neither).

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

## ROUND 2 (2026-11-21, the AI-INLINE-40 window) — the re-probe + the 39-site port

The four active tips re-probed at this round's start: **d53 `6d09fe61`**
(unchanged), **d75 `0e700d9a`** (unchanged, sounds-only), **g63 `2ea3e711`**
(batch 7 + the 281 correction + research/67 landed), **g64 `625c449e`**
(remainders 1-4 landed). f5 moved to `55dd8f9f` (NPC.cs wave D only — no
Projectile.cs claim change). The claim deltas against the round-1 census:

- **g64 now claims 3 of the 40**: `:35601`/`:35681` (remainder 1,
  `trail_650_scan_sparkle`/`trail_650_point_sparkle` in
  `projectile_ai_pets124.rs` — the 650's dust, d13-rowed) and `:34020`
  (remainder 2, `trail_813_opacity_tail`, the `:34018-34024` cite).
- **g63's doc-66 line-79 census row is corrected**: the "60 | 358" row lists
  `:29245/29284` but the C#'s `type != 1042` return at `:29234` precedes
  them — they belong to the **1042 ShimmerGunStream** (d6's row), NOT the
  358. g63's actual emitter (`projectile_trail_dust.rs:2134`) cites
  `:29167/29191` (`:29158-29196`) only, so no port collision — the e3
  `style60_shimmer_gun_trail_1042` takes the two.
- **The 33993/34005 pair**: g64 remainder 2's machine comment explicitly
  pins "`:33984-34016` stays the co-lane's unmodelled skip (the d11
  census)" — a cited SKIP, not a claim. e3 ports + WIRES it (the 578/579
  rows + `ai_vortex_ritual` live on this base).

### The 40-site ledger (line → kind/style → emitter → state)

| # | site(s) | kind(s) / aiStyle | e3 emitter | state | activating merge |
| - | ------- | ----------------- | ---------- | ----- | ---------------- |
| 1 | 27869 | 227 CrystalLeafShot / 43 | `style43_crystal_leaf_burst_227` | STAGED | d5 `projectile_ai_style43` (UNWIRED, no row) — its 8 kept NextFloat draws superseded |
| 2 | 28833 | 321 FlamingJack / 55 | `style55_flaming_jack_head_321` | STAGED | NO machine on any tip — the row+machine wave (d6 band follow-up) |
| 3-4 | 29245, 29284 | 1042 ShimmerGunStream / 60 | `style60_shimmer_gun_trail_1042` | STAGED | d6 `projectile_ai_guns` (rowed there, dust-skipped) |
| 5-8 | 31017, 31053, 31149, 31207 | 451 InfluxWaver / 81 | `style81_influx_spark_451` (one body, 4 call gates) | STAGED | d9 `projectile_ai_style81` |
| 9 | 31896 | 464 CultistBossIceMist / 86 | `style86_ice_mist_ring_464` | STAGED | d9 `projectile_ai_style86` |
| 10 | 31928 | 482 ClingerStaff / 87 | `style87_clinger_curtain_482` | STAGED | d9 `projectile_ai_style87` (zero-explicit-draw machine) |
| 11-12 | 32305, 32330 | 490 CultistRitual / 89 | `style89_ritual_ring_490` + `_puff_490` | STAGED | d9 `projectile_ai_style89` (kept-draws census REPLACED) |
| 13 | 32426 | 492 MagicLantern / 90 | `style90_lantern_spark_492` (per-cell) | STAGED | d9 `projectile_ai_style90` (the Next(4) && head stays its) |
| 14 | 32548 | 496 ShadowFlame / 91 | `style91_shadowflame_tail_496` | STAGED | d10 `projectile_ai_91_100` |
| 15 | 32958 | 518 CoinPortal / 94 | `style94_coin_portal_spark_518` | STAGED | d10 |
| 16 | 33015 | 524 IchorSplash / 96 | `style96_ichor_puffs_524` | STAGED | d10 |
| 17 | 33147 | 526 CultistBossParticle / 98 | `style98_cultist_particle_pair_526` | STAGED | d10 |
| 18-19 | 33993, 34005 | 578/579 Vortex markers / 108 | `vortex_dissolve_twin_578_579` | **WIRED** | live on this base (`ai_vortex_ritual_dust`; the g64 skip-cite resolved) |
| — | 34020 | 813 / 108 | g64 `trail_813_opacity_tail` | g64-CLAIMED | — |
| 20 | 34123 | 836 DandelionSeed / 112 | `style112_dandelion_burst_836` | STAGED | NO row on any tip (the g64 deferred ledger's latch-burst remainder) |
| 21-25 | 34507, 34512, 34516, 34524, 34531 | 612/953/978 / 117 flag58 | `style117_whip_explosion_612` | STAGED | d12 `projectile_ai_explosions` (kept-draws census REPLACED) |
| 26-28 | 34542, 34551, 34588 | 624 / 117 flag59 | `style117_guardian_explosion_624` | STAGED | d12 |
| — | 35601, 35681 | 650 / 124 | g64 `trail_650_*` | g64-CLAIMED | — |
| 29 | 35854 | 654+670 / 126 | `style126_column_rise` | STAGED | d13 `projectile_ai_geyser` (kept-draws census REPLACED) |
| 30 | 35914 | 654+670 / 126 | `style126_column_burst` | STAGED | d13 |
| 31 | 35932 | 654+670 / 126 | `style126_column_plume` | STAGED | d13 |
| 32 | 35956 | 654+670 / 126 | `style126_column_body` | STAGED | d13 |
| 33 | 35986 | 654+670 / 126 | `style126_column_tail` | STAGED | d13 |
| 34 | 36106 | 658 SandnadoHostileMark / 128 | `style128_sandnado_plume_658` | STAGED | d13 `projectile_ai_sandnado_mark` |
| 35-36 | 36157, 36170 | 658 / 128 | `style128_sandnado_orbits_658` | STAGED | d13 |
| 37 | 36427 | 672 DD2ElderWins / 132 | `style132_elder_wins_motes_672` | STAGED | d14 `ai_elder_wins` (`projectile.rs:10694+` at its tip) |

**37 of 40 ported** (34 STAGED + 3 WIRED), 3 g64-claimed. Commits:
`977b112d` (the AI_001 edges + helpers), `cefcb39c` (batch C, 8),
`e1f3813a` (batch D, 7), `a046c73d` (batch E, 5), `677f8d15` (batch F, 8),
`635e9e3a` (batch G, 5), `c9206e75` (batch H, 4). 17 draw-mirror witnesses
WRITTEN NOT RUN. The pool's `clone_dust` went generic over `DustRng` (the
g64 `853039bf` shape — identical change, keep-one at merge);
`special_arrows` gained `step_dust` (the g63 batch-1 seam — its 41 Hellfire
dust already rides a `step_dust` there: the two variants UNION at merge,
the 1006 call after the counter, the 41 call in the gravity arm).

### The AI_001-edges-2 ledger

| site | kind | e3 emitter | state | notes |
| ---- | ---- | ---------- | ----- | ----- |
| 56049 | 1006 ShimmerArrow | `ai001_shimmer_sparkle_1006` | **WIRED** | row + `special_arrows::step_dust` live here; the CloneDust White −0.3 twin, hue NextFloat BEFORE scale (argument order), fadeIn = scale+0.05 |
| 56748 | 981 SilverBullet | `ai001_silver_bullet_drip_981` | STAGED | no row on any pushed tip; the fade-chain slot AFTER the 265/355 arms, BEFORE the 357 trail (`:56746` between `:56728` and `:56753`); noLightEmittance + noGravity ×0.2 |

### The staged-emitter activation contract (per merge)

Every STAGED emitter above is drop-in at its machine's cited skip slot;
the machine lanes' kept explicit draws are REPLACED by the emitter call
(the g63 body-skip supersede convention) — EXCEPT the two zero-draw
machines (d9 style87, d9 style90's Next(4) head) where the emitter ADDS the
pool internals after the kept gates. The d13 geyser/sandnado and d12
explosions machines' per-tick RNG censuses (their module headers) are the
stream-parity contract: the emitter draws must land in exactly those
positions. Sentinel behaviour is the pool's (post-writes land on slot
6000). `customData`/`shader` writes are the documented dropped fields.

## The named-machines 87+36 future-wave assignment table

The two named-machine regions' uncited sites (round-1 census: 87 in
`:36809-54001`, 36 in `:57826-70641`) assigned per machine to the d-lane
whose pushed tip owns (or bands) the machine. **The contract per merge**:
when the owning d-lane merges, its machine's dust sites activate by the
e3 emitter pattern (argument draws first, pool internals, post-writes on
the returned slot) — either the d-lane ports them dust-wired (the g64
remainder shape) or a dust-lane follow-up drops emitters into its skip
slots (this round's shape). "main" = the machine is already on main; its
dust is a dust-lane follow-up, NOT a d-lane obligation. Site counts are
the fresh method-attribution scan (143 `void AI*` signatures).

| machine | sites | lines | owner lane (tip evidence) | dust-activation contract |
| ------- | ----- | ----- | ------------------------- | ------------------------ |
| AI_003_Boomerang | 15 | 38896-39617 | main (`ai_boomerang`) | dust-lane follow-up at its skip slots |
| AI_009_MagicMissiles | 5 | 43370-43416 | main (`ai_magic_missile`, projectile.rs:8298) | dust-lane follow-up |
| AI_009_MagicMissiles_Old | 5 | 43574-43635 | UNPORTED (the 491 legacy) — d1 band remainder | row+machine wave first |
| AI_010 | 13 | 57832-57940 | main (the falling-tile arms) + d1 band | the sand/falling-tile dust: dust-lane follow-up; the unported sub-arms d1's |
| AI_015_Flails_Dust | 7 | 44410-44470 | main (`ai_flail`) | dust-lane follow-up (the machine is NAMED for its dust) |
| AI_016_Bombs | 20 | 47035-47456 | main (`ai_bomb`/`ai_rocket`) | dust-lane follow-up; the e3 Kill_Bombs tail already covers the death half |
| AI_019_Spears | 14 | 45886-46048 | main (`ai_spear` + projectile_ai_spear) | dust-lane follow-up |
| AI_019_Spears_Old | 8 | 46490-46530 | UNPORTED legacy — no tip claims | row+machine wave first |
| AI_026 | 24 | 59182-63904 | main (the ai026_a/b/c slices) | dust-lane follow-up per slice |
| AI_029_GemStaffs (+ActuallyGemStaffs) | 9+2 | 37650-37813, 37851/37869 | main (`ai_gem_staff`) | dust-lane follow-up |
| AI_040_LeafBlower | 2 | 38046/38054 | main (`ai_leaf_blower`, draws threaded) | dust-lane follow-up at the petal-fade slots |
| AI_053_FrostHydra | 3 | 37279-37307 | main (cited) — verify which arm; d6 band | dust-lane follow-up |
| AI_061_FishingBobber_DoASplash | 1 | 53897 | main (projectile_bobber) | the splash dust: g15/d53 wet-family keep-one check first |
| AI_062 | 9 | 65134-66098 | main (the minion machines) | dust-lane follow-up per minion arm |
| AI_067_FreakingPirates | 9 | 49275-51162 | main (`ai_freaking_pirates`) | dust-lane follow-up |
| AI_075 | 13 | 66196-67365 | main (75 dispatch) — kinds 439/445/460/595/600/611/615/630/633/705/714/735/927 | dust-lane follow-up per kind arm |
| AI_099_2_Yoyos | 2 | 67872/67878 | main (`ai_yoyo`) | dust-lane follow-up |
| AI_111_DryadsWard | 2 | 41179/41197 | **d12** (`projectile_ai_misc` census) | d12 merge → emitters at its skips |
| AI_113_TargetSticker | 6 | 40641-40882 | **d12** | d12 merge |
| AI_120_StardustGuardian | 6 | 44668-44948 | main (`ai_stardust_guardian`) + d12 verify | dust-lane follow-up |
| AI_121_StardustDragon | 3 | 42852-43051 | **d12** | d12 merge |
| AI_123_FloatingTurrets | 4 | 37074-37123 | main (`ai_floating_turrets`) | dust-lane follow-up |
| AI_130_FlameBurstTower | 6 | 68140-68264 | main (130 dispatch) + **d14** band | d14 merge for the unported arms; main-half dust-lane |
| AI_131_FlameBurstShot | 1 | 68497 | main (131 dispatch) + d14 band | as above |
| AI_135_OgreStomp | 2 | 68732/68766 | **d14** | d14 merge |
| AI_136_BetsyBreath | 2 | 68817/68834 | **d14** | d14 merge |
| AI_137_LightningAura | 3 | 68971-69007 | main (137 dispatch) + **d14** | d14 merge for its arms |
| AI_140_MonkStaffT1 | 5 | 69475-69578 | **d14** | d14 merge |
| AI_142_MonkStaffT2And3 | 3 | 69862-69947 | **d15** (`projectile_ai_dd2`) | d15 merge |
| AI_143_MonkStaffT2Ghast | 1 | 70028 | **d15** | d15 merge |
| AI_144_DD2Pet | 3 | 70072-70147 | **d15** | d15 merge |
| AI_145_BookStaffStorm | 2 | 70418/70436 | **d15** | d15 merge |
| AI_147_Celeb2Rocket | 1 | 52940 | **d15** | d15 merge |
| AI_147_Explosion | 20 | 53138-53381 | **d15** | d15 merge (the biggest single-machine block) |
| AI_150_GolfClubHelper | 1 | 52460 | main (projectile_ai_golf) + d15 cite | dust-lane follow-up |
| AI_153_ToiletPop | 1 | 52225 | **d16** (`projectile_ai_kites` band) | d16 merge |
| AI_155_SpawnRopeIn | 2 | 52146/52148 | **d16** | d16 merge |
| AI_155_MysticSnakeCoil | 3 | 51969-52079 | **d16** | d16 merge |
| AI_157_SharpTears | 4 | 51435-51480 | main (`ai_sharp_tears`, rng-threaded) + d16 | dust-lane follow-up at its skips |
| AI_161_RapierStabs | 1 | 47654 | **d17** (projectile.rs:21764 at its tip) | d17 merge |
| AI_162_TigerPounce | 2 | 46744/46756 | **d17** (projectile.rs:21873) | d17 merge |
| AI_165_Whip | 26 | 47744-48252 | main (`ai_whip`) + d17 whip work | dust-lane follow-up (the largest named block; the whip census stubs are main's) |
| AI_168_FirstFractal | 1 | 46564 | **d17** (projectile.rs:7106, the hue-feed skip cited) | d17 merge — its GetFirstFractalColor hue feed is the dust colour seam |
| AI_169_Smolstars | 1 | 45781 | main (cited) | dust-lane follow-up |
| AI_171_HallowBossRainbowStreak | 1 | 45487 | main (cited) + d18 band | dust-lane follow-up |
| AI_177_IceWhipSlicer | 3 | 45240-45259 | **d18** (`projectile_ai_misc.rs:1833`) | d18 merge |
| AI_178_FireExplosion | 5 | 45274-45298 | **d18** (projectile.rs:17570) | d18 merge |
| AI_179_FairyQueenLance | 1 | 45040 | main (empress family) | dust-lane follow-up |
| AI_181_FairyQueenRangedItemShot | 1 | 43185 | main (projectile.rs:17797 dispatch) | dust-lane follow-up |
| AI_184_BadTorchLuck | 1 | 42810 | **d19** (`projectile_ai_sword_styles` census slice) | d19 merge |
| AI_186_PrincessWeapon | 2 | 42701/42731 | **d19** | d19 merge |
| AI_188_LightsBane | 1 | 42258 | main (`ai_lights_bane`) | dust-lane follow-up |
| AI_189_Volcano | 4 | 42307-42364 | main (`ai_volcano`) | dust-lane follow-up |
| AI_190_NightsEdge | 11 | 42034-42216 | **d19** (`projectile_ai_sword_styles.rs:631`) | d19 merge — the ground-bloom + sweep dust |
| AI_191_TrueNightsEdge | 5 | 41715-41922 | **d20** (the terra-chain verify: 973/985 owned) | d20 merge |
| AI_192_JuminoAnimation | 8 | 41525-41627 | **d20** (projectile.rs:17425) | d20 merge |
| AI_193_Flamethrower | 2 | 41374/41395 | **d20** (projectile.rs:17363) | d20 merge |
| AI_194_HorsemanPumpkin | 1 | 41329 | main (`ai_horseman_pumpkin`) + d20 cite | dust-lane follow-up |
| AI_195_JimsDrone | 1 | 41061 | **d20** (projectile.rs:17439) | d20 merge |
| AI_197_CeilingAndHoverTurret (+HandleTileCollision) | 1+1 | 40319, 40260 | **d20** (projectile.rs:18443/9005) | d20 merge |
| AI_198_Flint | 1 | 52312 | **d20** (projectile.rs:17457) | d20 merge |
| AI_199_MeteorOre | 2 | 40026/40032 | **d20** (projectile.rs:17391) | d20 merge |
| AI_200_BirdDroppings | 1 | 39994 | **d20** (projectile.rs:17376) | d20 merge |
| AI_202_TorchGodHelper | 1 | 39937 | **d21** (`projectile_ai_201_210`) | d21 merge |
| AI_203_StormLightning | 4 | 39756-39841 | **d21** | d21 merge |
| AI_204_Digtoise | 2 | 38712/38830 | **d21** | d21 merge (the Kill-head Digtoise pair 70753+ stays g65's) |
| AI_205_RemoteControlCar | 4 | 38112-38447 | **d21** | d21 merge |
| AI_208_GlacierFangShot | 2 | 36988/37036 | **d21** | d21 merge |
| AI_209_ChlorophyteClaymoreBladeSlam | 4 | 69742-69803 | **d21** | d21 merge |
| AI_210_YoyoShots | 1 | 36877 | main (`ai_yoyo_shot`) | dust-lane follow-up |

Unassigned residue (no machine port on any tip, no band owner): the two
`_Old` legacy machines (13 sites). The 358 WaterGun's `:29167/29191` stay
g63's (batch 6, d6-rowed); the 406's `:29205/29229` stay g64's ledger.
The AI_001 121-site region minus g63's 15 + e3's 2 edges = 104 main-stub
sites whose per-kind arms are the slice modules' — the g63 batch-5/7
pattern (a `step_dust` variant per slice) is their activation shape.
