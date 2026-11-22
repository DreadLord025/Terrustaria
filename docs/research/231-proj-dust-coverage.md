# 231 — PROJ-DUST master coverage census (W4-e3, coverage phase, 2026-11-21; REFRESHED same day, the AI-INLINE-40 round; SEALED same day, the RETIRE phase; number was 125 — the registry-150 rename EXECUTED at the seal, d125's empress-sheets-ledger 06:07:29 wins the 125 key per §7.1)

Status: **SEALED** (the RETIRE phase, below — the 34-STAGED activation
contracts re-verified against the moved d-lane tips, the `_Old` residue
resolved LIVE/DEAD, the Orch-61 residue lane-censused (d54/e4 own all 61,
the call-site wiring task-seeded), the g63 doc-66 receipt published, the
g64 `clone_dust` keep-one confirmed BOTH ways; no new sites claimed after
this seal). CENSUS + two
window claims (the `:78227-78508` Kill_Bombs tail,
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
| W4-g64 `proj-trail-dust-2` | `9d9d89ac1` (retire) | per-kind TRAIL, kinds ≥401 | 41 kinds / 5 batches + the wiring wave + remainders 1-4: the style-124 machine dust-wired (**claims `:35231/35295/35601/35681`**), the 813 four-phase bodies (**claims `:34020`** + the `:33951-33983` dissolve), the 596 row, the e3/g63 keep-one refresh; FINAL 1/2 (the d13 dup adjudication + `trail_503` ACTIVATED via the e3 row) + its own RETIRE; doc renumbered 66 → **196** (the d54 collision registry) |
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
   below; the RETIRE pass re-verified all 34 contracts at the moved tips —
   321/836 are now d6/d12-rowed, see the RETIRE section). The g63 doc-66 census row "60 | 358 (`:29167/.../29245/29284`)"
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
   superseded; the RETIRE pass resolves the `_Old` residue — 8 DEAD
   cite-dropped, 5 LIVE task-seeded to d1).
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
  wave; the e3 arm is the rowed survivor on this branch. (RETIRE receipt:
  g64 FINAL 2 `2e8ea03fa` executed exactly this — its 503 row is the e3
  `80d47a568` constants-copy and its keep-one names the e3
  `ai_falling_star_family` as the fuller survivor; the g64 503 half retires
  into it at merge.)
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
| 2 | 28833 | 321 FlamingJack / 55 | `style55_flaming_jack_head_321` | STAGED | **d6 `ai_flaming_jack`** (RETIRE: row+machine NOW on d6 `7edfcd495` — projectile.rs:4731, the `:28832-28838` zero-explicit-draw skip; the emitter ADDS the pool internals, the style87 shape) |
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
| 20 | 34123 | 836 DandelionSeed / 112 | `style112_dandelion_burst_836` | STAGED | **d12 `ai_dandelion_seed`** (RETIRE: row+machine NOW on d12 `c98bd8937` — projectile.rs:10080, the `:34121-34126` latch-burst zero-explicit-draw skip; the emitter ADDS the pool internals) |
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
| AI_009_MagicMissiles_Old | 5 | 43574-43635 | **LIVE legacy** (RETIRE census: called at `:43199` under `type == 491` in `AI_009_MagicMissiles`; the 491 FlyingKnife row unported on every tip) — d1 band remainder | row+machine wave first (the task seed below) |
| AI_010 | 13 | 57832-57940 | main (the falling-tile arms) + d1 band | the sand/falling-tile dust: dust-lane follow-up; the unported sub-arms d1's |
| AI_015_Flails_Dust | 7 | 44410-44470 | main (`ai_flail`) | dust-lane follow-up (the machine is NAMED for its dust) |
| AI_016_Bombs | 20 | 47035-47456 | main (`ai_bomb`/`ai_rocket`) | dust-lane follow-up; the e3 Kill_Bombs tail already covers the death half |
| AI_019_Spears | 14 | 45886-46048 | main (`ai_spear` + projectile_ai_spear) | dust-lane follow-up |
| AI_019_Spears_Old | 8 | 46490-46530 | **DEAD code** (RETIRE census: zero callers tree-wide — the `:46222` definition is the only occurrence in `decompiled/Terraria/*.cs`) | CITE-DROP — the 8 sites leave the uncited floor (the RETIRE section below) |
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

Unassigned residue — RESOLVED by the RETIRE census (below): of the two
`_Old` legacy machines (13 sites), `AI_019_Spears_Old`'s 8 are DEAD code
(zero callers — cite-drop) and `AI_009_MagicMissiles_Old`'s 5 are LIVE
(the 491 dispatch at `:43199`) — task-seeded to the d1 band's row+machine
wave. The 358 WaterGun's `:29167/29191` stay
g63's (batch 6, d6-rowed); the 406's `:29205/29229` stay g64's ledger.
The AI_001 121-site region minus g63's 15 + e3's 2 edges = 104 main-stub
sites whose per-kind arms are the slice modules' — the g63 batch-5/7
pattern (a `step_dust` variant per slice) is their activation shape.

## RETIRE (2026-11-21, the sealing pass — the AI-inline family declared DONE)

**Declaration: the W4-e3 AI-inline family is DONE.** The two windows this
branch claimed — the `:78227-78508` Kill_Bombs dust tail (15 sites) and the
AI-INLINE-40 + AI_001-edges-2 window (37+2 sites) — are fully ported (the
ledgers above), witnessed (WRITTEN NOT RUN, code-only) and contract-cited.
No further site claims ride this branch; the residue is assigned below.
This doc is SEALED: later tip movement is recorded as receipts, not
re-openings.

### (1) The 34-STAGED activation contracts — re-verified at the latest tips

All seven cited machine lanes were re-fetched and their machines + dust-skip
censuses re-read at their CURRENT tips (2026-11-21 retire fetch):

| lane | retire tip | machine evidence (module/fn + the dust census) |
| ---- | ---------- | ---------------------------------------------- |
| d5 `e46d9eb7f` | `projectile_ai_style43.rs` | still `#![allow(dead_code)] // UNWIRED` (:51 — the 227 row + dispatch are d5's own wiring step); the `:27873` 8×NextFloat kept-draws census intact (:106-107) — the e3 emitter supersedes it at the d5 merge |
| d6 `7edfcd495` | `projectile_ai_guns.rs` + `ai_flaming_jack` | the 1042 row (:2874) + the 6×[3 Next(2) + Next(6)] + Next(4)-gated Next(6) trail census intact; **NEW: the 321 row (:1057/:2858) + `ai_flaming_jack` (projectile.rs:4731, dispatch-55 arm :17942) — the `:28832-28838` skip rolls NOTHING** |
| d9 `7a418c636` | `projectile_ai_style81/86/87/89/90.rs` | all five present; style81's consumed-draws census (:31-34), style86's 24-NextDouble census (:30-32), style87's ZERO-explicit-draw census (:29), style89's kept draws (:38-40), style90's Next(4)-head-kept census (:38-40) — all intact |
| d10 `e1519df75` | `projectile_ai_91_100.rs` | present; the per-style censuses intact — 91 ZERO draws (:46), 94 the tick-40 Next(3) (:56), 96 the 2× Next(3) gates (:65), 98 the 2× NextFloat scales (:67) |
| d12 `c98bd8937` | `projectile_ai_explosions.rs` + `ai_dandelion_seed` | the flag58/flag59 per-family RNG censuses intact (module header :24-36); **NEW: the 836 row (:1090/:3697) + `ai_dandelion_seed` (projectile.rs:10080, dispatch-112 arm :19568) — the `:34121-34126` latch burst is SKIPPED as all-NewDust-internals, ZERO explicit draws** |
| d13 `462c86097` | `projectile_ai_geyser.rs` + `projectile_ai_sandnado_mark.rs` | both present; the by-construction RNG censuses intact (geyser :10 + the CensusRng witness :277; sandnado_mark :9 + :164) |
| d14 `eaa05ae9e` | `ai_elder_wins` (projectile.rs:10720) | present (moved from :10694 at the round-2 read); the taken-branch 5-draw census intact (the doc-comment :10694-10718) |

**The two flag flips** (the round-2 "NO row on any tip" flags):

- **321 FlamingJack — NOW ROWED on d6.** The round-2 ledger's "row+machine
  wave (d6 band follow-up)" has LANDED on the d6 branch itself. Contract
  update: `style55_flaming_jack_head_321` activates at the **d6 merge**,
  dropped into `ai_flaming_jack`'s `:28832-28838` skip (inside the C#'s
  `ai[0]` window, after the target machine). The d6 machine keeps ZERO
  explicit draws there, so the emitter ADDS the pool internals (the
  style87 add-shape, NOT the replace-shape).
- **836 DandelionSeed — NOW ROWED on d12.** Same flip: row + machine +
  dispatch all on the d12 tip. Contract update:
  `style112_dandelion_burst_836` activates at the **d12 merge**, dropped
  into `ai_dandelion_seed`'s `:34118-34127` latch block (the caller owns
  the `localAI[0]` latch; the emitter fires INSIDE it). ZERO explicit
  draws — the add-shape again. The g64 deferred-ledger latch-burst
  remainder is thereby owned by d12's machine + this emitter.
- **981 SilverBullet - STILL unrowed** (the full 345-branch remote scan;
  re-verified at the seal's 358-tip fetch - zero `SILVER_BULLET`/`981 =>`
  hits in any pushed tip's `projectile/data.rs`; re-verified AGAIN at the
  FINAL seal's 360-tip fetch — zero `981 =>`/`SILVER_BULLET` hits, and
  zero 981 code hits in d75/d1's arms).
  `ai001_silver_bullet_drip_981` stays STAGED for the bullet-row wave; its
  fade-chain slot citation (`:56746` between `:56728` and `:56753`) stands.
  **The row-lane task seed is FINAL — see "The FINAL seal receipts" (7)
  below: owner d1, the aiStyle-1 bullet sibling wave** (the i18
  research/63 §5 OUT-list sibling; the d75 first-120 census is a
  SOUND-range census — its `:41968` boundary EXCLUDES the `:56748` dust
  site; the 981's only sound sites are the Kill-tail `:76043` d77/f1
  deferred-unrowed rows).
- 227 CrystalLeafShot — still unrowed even on d5 (its module's own
  `UNWIRED` marker); the d5 row+dispatch step activates machine AND
  emitter together. Unchanged.

The remaining 31 staged emitters' contracts are UNCHANGED — every cited
machine exists at its lane's latest tip with its census intact; the
replace-vs-add split in "the staged-emitter activation contract" above
gains two add-shape members (321, 836).

### (2) The `_Old` legacy residue — the LIVE/DEAD caller census

The round-1/2 "13 unassigned sites" were the two `_Old` machines. The
caller census (the full `decompiled/Terraria/*.cs` scan, not just
Projectile.cs — the private methods cannot be called cross-file anyway):

- **`AI_009_MagicMissiles_Old` (`:43427`, 5 sites `:43574-43635`) — LIVE.**
  One caller: `AI_009_MagicMissiles`'s head (`:43197-43200`): `if (type ==
  491) { AI_009_MagicMissiles_Old(); return; }`. The 491 FlyingKnife
  (`ProjectileID.cs:1536`) is UNROWED on every pushed tip (the 345-branch
  scan: only the 16 MagicMissile row exists; re-verified at the seal's
  358-tip fetch — zero `FLYING_KNIFE`/`= 491` hits in any pushed tip's
  `projectile/data.rs`). **Disposition: task seed,
  owner lane d1** (the aiStyle-6/9 band remainder): the 491 row +
  `AI_009_MagicMissiles`'s type-491 head + the `_Old` body ride one wave;
  its 5 dust sites activate by the e3 emitter pattern in that wave's
  dust follow-up. NOTE the `_Old` body itself branches on `type == 16`
  too (`:43434`) — dead for the only reachable kind 491, kept verbatim
  when ported (the census-F2 convention).
- **`AI_019_Spears_Old` (`:46222`, 8 sites `:46490-46530`) — DEAD.**
  ZERO callers anywhere in the decompiled tree (the definition line is
  the only occurrence; the live spear dispatch `:25310` calls
  `AI_015_Flails`/`AI_019_Spears`, never the `_Old`). **Disposition:
  CITE-DROP** — the 8 sites leave the coverage floor; no port owes them.
  The named-machine uncited count 87 → **79** (the residue 13 → 5, all
  LIVE, all task-seeded above). The sibling `AI_015_Flails_Old`
  (`:44205`) is likewise dead (zero callers) but carries NO dust sites —
  it never entered this census.

### (3) The g63 doc-66 census-error receipt (mutual, published here)

**e3 → g63**: the round-2 correction stands, re-verified against g63's
tip `2ea3e7110` at the RETIRE read (g63 has since moved to `1c52d5ee3`
and EXECUTED the fix — the closing receipt is (9) below):

- The error: `docs/research/66-proj-trail-dust-claims.md:79` (g63's tip)
  rows `| 60 | 358 (:29167/29191/29205/29229/29245/29284) | last prism |`.
  The last four lines are NOT the 358's: the C#'s kind gates are
  `type == 358` (`:29158-29196` — `:29167/29191` only), `type == 406`
  (`:29197-29233` — `:29205/29229`), then **`if (type != 1042) return`
  (`:29234-29237`)** before `:29245/29284` — those two are the **1042
  ShimmerGunStream's** (dust 324, not the 358's dust 211).
- No port collision: g63's actual emitter
  (`projectile_trail_dust.rs:2134-2135` at its tip) cites
  `:29167/29191` (`:29158-29196`) ONLY — the code was always right; only
  the doc row over-claims. The 406's `:29205/29229` are g64's ledger
  (kinds ≥401).
- The correction's port: e3 `style60_shimmer_gun_trail_1042`
  (`projectile_dust.rs:1641`, STAGED for the d6 merge) owns
  `:29245/29284`; the d6 machine's per-tick 1042 census (the 6×[3
  Next(2)+Next(6)] + Next(4)-gated Next(6)) is its wiring contract.
- **g63-lane read request**: fix the `:79` row to
  `| 60 | 358 (:29167/29191) | ... |` (the 406 pair to the g64 ledger,
  the 1042 pair to e3's emitter) at g63's next touch. e3 does not edit
  g63's branch. This receipt + research/231's (was 125) round-2 section are the
  mutual record; the g63 acknowledgement (if any) lands on g63's branch.

### (4) The g64 `clone_dust` genericization — keep-one CONFIRMED

The e3 pool change (batch B, `977b112d`) and g64's `853039bfcc` are the
SAME change: `DustPool::clone_dust` generic over `R: DustRng` (e3 HEAD
`particles.rs:692`, g64 tip `:699`). Verified by `git diff HEAD
origin/task/W4-g64-proj-trail-dust-2 -- crates/tr-core/src/particles.rs`:
**the function bodies are identical — the only delta is g64's three
doc-comment additions** (the W4-g64 attribution lines on
`new_dust_perfect`/`new_dust_direct`/`clone_dust`). **Keep-one at merge:
take EITHER body verbatim; prefer g64's doc-comments** (they document the
seam for both lanes' callers). The sentinel guard, the field-copy list
and the dropped `shader`/`customData` are identical on both sides. No
reconciliation edit owed by either lane. **RETIRE receipt (g64 moved
`625c449e` → `9d9d89ac1` mid-seal): g64's own RETIRE `9d9d89ac1` adjudication
(1) independently CONFIRMS this keep-one** — its comment-stripped span diff
of `DustRng`/`new_dust_perfect`/`new_dust_direct`/`clone_dust` is
byte-identical CODE, doc-comment wording only (the d54 bible s8.3 step-2
verdict). Both lanes' receipts agree; the f50 `BlastRng` adapter is per-lane
glue, no adapter keep-one owed. The re-verified `git diff HEAD
origin/task/W4-g64-proj-trail-dust-2 -- crates/tr-core/src/particles.rs` at
the moved tip still shows the three doc-comment hunks ONLY.

### (5) The Orch-61 particle-orchestrator residue — lane census + disposition

The d54 bible s8.2's fresh C# spawn census lists **Orch 61** beside the
Projectile 1195. Those 61 are `GameContent/Drawing/ParticleOrchestrator.cs`
sites (16 NewDust + 37 NewDustPerfect + 8 NewDustDirect, 8 CloneDust
siblings riding their parents) — a DIFFERENT file from this doc's
Projectile.cs census. **Which lane owns: the world-dust pair, d54 + e4 —
all 61 already landed as LATENT emitters.**

- **d54 `38bfd1d22`** (research/67 at its tip): orch sites **1-40**
  (`:423-2582`, ArmletOfRuinRecharge → CopperSlimeEffect), batches 5-8
  `399fbb61`/`6b0b5dee`+`dc299f83`/`965e9612`/`5fe70387` —
  `orch_dust.rs`/`orch_dust_lightning.rs`/`orch_dust_boulder.rs`/
  `orch_dust_town.rs`, 21 witnesses.
- **e4 `2a79551ab`** (research/66 at its tip): orch sites **41-61**
  (`:2653-3920`) `fc2f64a22`+`e0d6d870c` — `orchestrator_dust.rs`, 9
  witnesses — plus Mount 1-19 (lane complete 40/40).
- The re-derived site scan at this seal (69 family hits = 61 sites + 8
  CloneDust) matches both claim docs line-for-line. NO site unclaimed;
  **no port-small owed by any lane**.

**The e3-side residue is the CALL-SITE wiring, not the bodies.** A fresh
`ParticleOrchestrator.` scan of Projectile.cs finds **57 call sites**;
Main.cs adds **3** (`ReforgeItemInReforgeSlot` `:41911/41916/41923`).
Attributed per enclosing method:

| enclosing method | sites | lines | dust-bearing orchestra types → owner |
| ---------------- | ----- | ----- | ------------------------------------ |
| `Damage_PVE_Inner` | 11 | 12970-14266 | FormatC/NightsEdge/TrueNightsEdge/TerraBlade/Excalibur/TrueExcalibur/MoonLordWhipHit×2 (dust: e4/d54); RainbowRodHit + StardustPunch×2 (e4) |
| `TryMakingSnappingStoneStrike` | 2 | 14355/14363 | SnappingStoneMelee/Summon (d54) |
| `SpawnHitVisuals` | 3 | 14407-14424 | `SpawnLightningExplosionDust` direct call :14407 (d54) + BlueLightningSmallLong + CattivaHit (dust-free) |
| `SpawnMoonLordWhipProc` / `EmitBlackLightningParticles` | 2 | 14928/15122 | dust-free (MoonLordWhipEye / BlackLightningHit) |
| `HandleMovement` | 3 | 18893/19250/19539 | Shadowbeam pair :18893 + PooFly :19250 dust-free; PaladinsHammerShockwave :19539 (d54) |
| rainbow-boulder / fake-fish helpers | 8 | 20181-20853 | RainbowBoulder1/4 (d54); LakeSparkle/FakeFish/FakeFishJump dust-free; the :20850-20853 Ping/Push/Magnet trio are state fns, no spawns |
| `AI()` inline | 5 | 26111-34631 | ShimmerBlock :26111 + GasTrap :32574 + MushroomBoiExplosion :34631 + LeafCrystalShot :27884 dust-free; BarnacleShotFiring :26665 (d54) |
| named machines | 12 | 39789-57819 | StormlightningWindup×2 :39789/39799, PrincessWeapon×3 :42757-42778 (e4), BlackLightningSmall :47840, ShimmerBlock :49746, RainbowBoulderPetBounce :49813 (d54 `rainbow_boulder4(pet)`), MushroomBoiTargetFound :51223, CattivaHit :51284, DeadCellsFlint :52322 (d54), HarpyCharm/PhoenixQuiver/SeraphNecklace :57819 — all except PrincessWeapon/RainbowBoulderPetBounce/DeadCellsFlint dust-free |
| `Kill()` | 10 | 70739-76875 | the 433/255/260/290/294 beam-kind switch :70739 dust-free; SilverBulletSparkle :70764/76044 + ArcSurge :70781 + StormLightning :70807 (d54) + RainbowRodHit :71244/71316 (e4) + StellarTune :71620 + ShimmerArrow :76060 (e4) + DeadCellsBarrelExplosion :76875 (d54) |
| `SpawnLightningLiquidElectrificationParticles` | 1 | 78551 | StormlightningWindup, dust-free — inside the e3 Kill-tail window's tail method, cited skip |

**Disposition: TASK SEED — the orchestrator wiring wave** (the d54/e4
world-dust lanes' own follow-up, the d54 landing record's "the wiring wave
folds them into `SpawnParticlesDirect`'s port"). The port today: the net
relay round-trips and drops (`app/netmodules.rs` `module_id::PARTICLES`,
both tips), d60's `tile_draw_particles.rs` queues `ParticleRequest` bytes,
`prefix.rs` S1-P-i45 drains the reforge tails count-only (272 + 16). The
wave ports `SpawnParticlesDirect` (`:162-416`) + the request/broadcast
front (`:52-90`), routes every call site above to the landed emitters, and
folds the prefix count-only drains into e4's real `rainbow_rod_hit` (the
e4 doc's own merge note). NOT e3's lane: zero of the 61 sites are
Projectile.cs sites, and the e3 windows never claimed a call-site wire.

**The dust-FREE spawners still consume stream draws** — the wiring wave
must keep them as ordered `next_*` drains even where no dust spawns (the
`prefix.rs` convention). The Projectile.cs-reachable ones, censused here
as the seed's draw contracts (visuals dropped, `ParticlePool` requests
RNG-free):

| spawner | lines | draw contract |
| ------- | ----- | ------------- |
| `Spawn_GasTrap` | 2911-2953 | the Item16 cue first (sound lane); 12×[NextFloat + RotatedByRandom 2×NextDouble + Next(20) + NextFloat] then 10×[NextFloat + 2×NextDouble + Next(30) + NextFloat] — 12×5 + 10×5 = 110 |
| `Spawn_NightsEdge` | 3205-3260 | ZERO draws (two 3-sparkle loops, constants only) |
| `Spawn_SilverBulletSparkle` | 3262-3284 | **5**: the DISCARDED angle NextFloat :3264 + scale NextFloat :3266 + DISCARDED NextFloat :3267 + Circular(4,4) 2 :3269 (the `new Vector2(f)` uniform is ONE draw, both components) |
| `Spawn_LeafCrystalPassive`/`Shot` | 3089-3137 | Passive: 1 head NextFloat :3092 (3-sparkle loop draw-free); Shot: ZERO draws (:3120 hsl uses UniqueInfoPiece) + the AddLight :3135 (light lane) |
| `Spawn_BlackLightningSmall` | 3494-3533 | 2 head (NextFloat + Next(1,3)) then num2∈{1,2} iterations × [NextFloatDirection + NextFloat + Circular(12,12) 2 + 2 colour NextFloats + Next(4) + 1 gated NextFloat] = 8+g per iteration |
| `Spawn_BlackLightningHit` | 3535-3574 | 1 head NextFloat, 7 iterations × the same 8+g body |
| `Spawn_BlueLightningSmallLong` | 3576-3597 | ZERO own draws; delegates ⌈len/25⌉ `Spawn_LightningSmall` calls when len ≥ 40 |
| `Spawn_StellarTune` | 3655-3681 | 1 head NextFloat + 5 × [NextFloatDirection + Circular(12,12) 2 + NextFloat + Next(2) + gated NextFloat + NextFloat scale] |
| `Spawn_CattivaHit` | 3745-3776 | 1 head NextFloat + 6 × [NextFloatDirection + Circular(6,6) 2 + NextFloat + Next(2) + gated NextFloat + NextFloat scale] |
| `Spawn_StormLightningWindup` | 974-1024 | 2 head (NextFloat + Next(1,3)) then num2 × [Next(7,11) + NextFloatDirection + NextFloat + Circular(6,6) 2 + Next(3) gate] = 7 per iteration, the twin particle copies draw-free |
| `Spawn_BestReforge` | 3069-3087 | 8 × Circular(3,4) = 16 (the `prefix.rs:678` drain already ports this verbatim) |

(`Spawn_RainbowRodHit`'s 272-draw contract stays `prefix.rs:640`'s + e4's
emitter; the HarpyCharm/LakeSparkle/FakeFish/PooFly/ShimmerBlock/ArcLightning
dust-free spawners are d54's research/67 dust-free list — their argument
draws ride the same wave.)

### (6) The sealing state

- 1195-site census: windows 1 (g65 remainder), 4 (the per-machine table,
  now 79+36 uncited after the cite-drop) stay assigned; windows 2/3/5 are
  e3-DONE; the Kill-tail 15 are e3-DONE. The Orch-61 (a DIFFERENT file) is
  d54/e4-DONE bodies + a task-seeded wiring wave (section 5).
- The e3 branch `task/W4-e3-dust-sites-proj-2` retires here: no further
  commits beyond the receipts. The merge order is the activation order —
  each d-lane merge activates its cited emitters per the contracts above.
- Code-only throughout: no cargo/rustc/tests/rustfmt/graphify invoked;
  all evidence is git-tip reads + C# line reads.

## The FINAL seal receipts (2026-11-21, the e3f dispatch — the five closing items + the registry rename)

Fresh 360-tip fetch at this round; every hash below re-read at THAT fetch.
The doc number changed **125 → 231** (the registry-150 §3 rename EXECUTED:
d125's `125-empress-sheets-ledger.md` first-add 06:07:29 `2dfe026d2` wins
the 125 key per the §7.1 round-3 correction; d38 → 293, d39 → 230, this
doc → **231** — the assignment was already published at d54 `72226e500`/
`51a28d09e`; all in-tree refs updated, the g63 doc-66/240 rows already
cite "e3 research/125 → 231 per registry-150").

### (7) The FINAL activation-order table — the 34 STAGED, for the merger

**The merge order IS the activation order** — the emitters are drop-in at
their machines' cited skip slots and activate at the OWNING LANE'S MERGE,
in whatever order the merger takes the lanes; there is NO cross-lane
dependency among the 34 (each lane's row + machine + census ride its own
branch). Per-lane table (tips re-verified at this seal's fetch — ALL
SEVEN unchanged since the RETIRE re-probe):

| merge | tip (this fetch) | emitters activated | sites | shape at merge |
| ----- | ---------------- | ------------------ | ----- | -------------- |
| **d5** | `e46d9eb7f` | `style43_crystal_leaf_burst_227` | 27869 (1) | REPLACE the module's 8 kept call-site NextFloat draws (the `:27873` census, module :106-107); the 227 row + dispatch are d5's OWN wiring step (module still `#![allow(dead_code)] // UNWIRED` :51) — row, machine AND emitter activate together |
| **d6** | `7edfcd495` | `style55_flaming_jack_head_321` | 28833 (1) | **ADD** (the flag flip): row (:1057/:2858) + `ai_flaming_jack` (projectile.rs:4731, dispatch-55 :17942) now on the tip; the `:28832-28838` skip rolls NOTHING — the emitter adds the pool internals inside the C#'s `ai[0]` window after the target machine (the style87 shape) |
| **d6** | `7edfcd495` | `style60_shimmer_gun_trail_1042` | 29245/29284 (2) | REPLACE the 1042 per-tick census in `projectile_ai_guns` (6×[3 Next(2)+Next(6)] + Next(4)-gated Next(6) — the emitter draws the SAME sequence and spawns the bodies; keeping both double-draws). The g63 doc-66 correction receipt — §3 + (9) below |
| **d9** | `7a418c636` | `style81_influx_spark_451` (4 call gates) | 31017/31053/31149/31207 (4) | REPLACE the launch/pulse SelectRandom+Next(6) census; the homing arm's OUTER Next(3) gate stays the caller's (drawn BEFORE the pick); the strike arm (`:31201`) has NO Next(6) — its noGravity is unconditional (the emitter's `gate_next6=false` call) |
| **d9** | `7a418c636` | `style86_ice_mist_ring_464` | 31896 (1) | REPLACE the 24-NextDouble census (module :30-32); the caller keeps `ai[1]!=1` + the 45-tick-kill RETURNS-before-dust order |
| **d9** | `7a418c636` | `style87_clinger_curtain_482` | 31928 (1) | **ADD** — the ZERO-explicit-draw machine (module :29); pool internals are the whole stream cost |
| **d9** | `7a418c636` | `style89_ritual_ring_490` + `_puff_490` | 32305/32330 (2) | REPLACE the burst/puff censuses (module :38-40); the caller gates `ai[0]>=60` AFTER the increment for the puff |
| **d9** | `7a418c636` | `style90_lantern_spark_492` (per-cell) | 32426 (1) | **ADD** the body after the kept gates — the per-cell Next(4) `&&` head stays the caller's, consumed for EVERY cell (module :38-40); the scan walk + `is_tile_spelunkable` are the machine's |
| **d10** | `e1519df75` | `style91_shadowflame_tail_496` | 32548 (1) | **ADD** — the style-91 machine keeps ZERO draws (module :46); the caller gates `scale < 1` |
| **d10** | `e1519df75` | `style94_coin_portal_spark_518` | 32958 (1) | REPLACE the taken branch's RotatedByRandom pair (module :56); the caller KEEPS the `alpha < 90` gate + the Next(3) draw itself (the C# && order — the gate draws every tick, the emitter fires only on the taken branch) |
| **d10** | `e1519df75` | `style96_ichor_puffs_524` | 33015 (1) | REPLACE the 2×Next(3) gates census (module :65); the caller owns the ai[0] +=0.6 + the >500 kill (no return) |
| **d10** | `e1519df75` | `style98_cultist_particle_pair_526` | 33147 (1) | REPLACE the 2×NextFloat scale census (module :67) |
| **d12** | `c98bd8937` | `style112_dandelion_burst_836` | 34123 (1) | **ADD** (the flag flip): row (:1090/:3697) + `ai_dandelion_seed` (projectile.rs:10080, dispatch-112 :19568) now on the tip; the `:34121-34126` latch burst is skipped as all-NewDust-internals — the caller owns the `localAI[0]` latch, the emitter fires INSIDE it |
| **d12** | `c98bd8937` | `style117_whip_explosion_612` | 34507/34512/34516/34524/34531 (5) | REPLACE the flag58 kept-draws census (module header :24-36) — the per-family RNG census is the stream-parity contract |
| **d12** | `c98bd8937` | `style117_guardian_explosion_624` | 34542/34551/34588 (3) | REPLACE the flag59 census (same header); the SIX pre-loop draws + the 200×135 ring order pinned |
| **d13** | `462c86097` | `style126_column_rise`/`_burst`/`_plume`/`_body`/`_tail` | 35854/35914/35932/35956/35986 (5) | REPLACE `projectile_ai_geyser`'s per-tick census (module :10 + the CensusRng witness :277): state-0 2×Next(5), burst/plume SelectRandom+NextFloat+sign-gated Next(4), body Next(3)<2 + Next(6)<3, tail NextFloat gate — the emitter draws land in EXACTLY those positions |
| **d13** | `462c86097` | `style128_sandnado_plume_658` + `_orbits_658` | 36106/36157/36170 (3) | REPLACE `projectile_ai_sandnado_mark`'s NextFloat-order census (module :9 + :164): amount FIRST then the X-lerp, the >−1.2-gated vel.X NextFloat then the unconditional vel.Y; the two orbit loops' rotation asymmetry (loop 1 RotatedBy(rotation), loop 2 NOT) |
| **d14** | `eaa05ae9e` | `style132_elder_wins_motes_672` | 36427 (1) | REPLACE the taken-branch 5-draw census (`ai_elder_wins` projectile.rs:10720, doc-comment :10694-10718); count = 1+(int)(−vel.Y) truncating cast, Next(2) gate per iteration |

Every STAGED entry of the round-2 ledger is rowed above — its #1-#37
numbering minus the WIRED #18-19 pair (the family's sealed "34-STAGED"
label; the multi-site entries #3-4/#5-8/#11-12/#21-25/#26-28/#35-36
carry their full site lists in the table). Shape split: **REPLACE** —
the machine's kept explicit draws are superseded by the emitter call
(the g63 body-skip convention) — EXCEPT the sealed contract's four
**ADD** members (d9 style87, d9 style90's body after the kept Next(4)
head, + the two flag flips 321/836); d10 style91's machine likewise
keeps ZERO explicit draws (its module :46 census — nothing to replace,
the emitter adds the internals after the caller's `scale < 1` gate).
Sentinel behaviour is the pool's (post-writes land on slot 6000);
`customData`/`shader` writes are the documented dropped fields. The
remaining STAGED outside the 34: the Kill-tail nine (773 scarab five /
588 confetti three / 1088 acorn one — the row/host wave, this doc's
coverage-phase section) + the 981 drip (the d1 bullet wave, (8) below)
+ the 227 (d5's own wiring step, above).

### (8) The two unrowed-kind task seeds — FINAL dispositions

- **981 SilverBullet → owner lane d1, the aiStyle-1 bullet sibling
  wave.** The row-lane question is CLOSED: the 981's dust site
  (`:56748`, AI_001) is NOT in d75's first-120 census — that census is
  the SOUND-site range #1-#120 ending at `:41968` (d75's own boundary,
  ANALYSIS Part 2/5 + research/32 at `b59e3a874`); `:56748` sits past
  it. The d75-tip 981 hits are the item lane only (`itemdata.rs:444`
  item 278 → projectile 981; `projectile_id.rs:993` the name row) —
  ZERO 981 hits in its projectile arms (grep exit-1 over
  projectile.rs/data.rs/special_arrows.rs/projectile_special_bullets.rs).
  The 981's SOUND sites are the Kill-tail `:76040-76048` (Item10 +
  SilverBulletSparkle) — d77 `5f10560b2` + f1 `3065492b0` both list
  `:76043` (981) in their deferred-unrowed ledgers. The ROW itself is
  the i18 research/63 §5 OUT-list sibling ("The 302 SniperBullet / 981
  SilverBullet / 110 BulletSnowman siblings — other tasks' kinds; 110
  is ai001_c's"): 110 landed via S1-D-3q/S1-D-5 slice C on **d1**, and
  d1's band (aiStyle 1-10) owns the remaining aiStyle-1 bullet rows.
  **The wave**: the 981 row (`Projectile.cs:9340-9353` — 4×4, aiStyle 1,
  friendly, pen 1, light 0.5, alpha 255, scale 1.2, timeLeft 600,
  ranged, extraUpdates 1) + the AI_001 fade-chain arm slot (`:56746`
  BETWEEN the 355 arm `:56728-56745` and the 357 trail `:56753` — the
  e3 `ai001_silver_bullet_drip_981` STAGED emitter is the drop-in; the
  1-in-6 Next(6) gate draws EVERY tick the kind matches) + the sibling
  sites the row unblocks: the Damage_PVE ×3-vs-Wyvern-head gate
  (`:13385-13388`), the ProjLight 0.7/0.7 arm (`:16343-16347`), the
  GetAlpha White (`:78927-78930`), the Kill-tail sound+sparkle
  (`:76040-76048` — d77/f1's cue + the orchestrator wiring wave's
  SilverBulletSparkle 5-draw contract, §5).
- **491 FlyingKnife → owner lane d1, the row+machine wave (CONFIRMED
  at the moved d1 tip `212516a82`).** The d1-band evidence: (a) the
  `ai_magic_missile` machine doc at its tip (projectile.rs:8190/:8197)
  pins "the `type == 491` `_Old` variant has no ported row … the
  `_Old` arm (491) … SKIPPED" — the machine is d1-band-owned and its
  491 head is the cited gap; (b) the falling-retire wave (d1's
  `projectile_ai_falling.rs`, waves 3a-3c `a2dcdb152`/`e001d9215`/
  `e47181798` + the hygiene `212516a82`) rowed the aiStyle-10 families
  and RETIRED its active reads — the 491 row is NOT among them (zero
  `FLYING_KNIFE`/`491 =>` hits at the tip, the 360-tip scan re-run);
  the band remainder is exactly the aiStyle-6/9 residue this seed
  names. **The wave**: the 491 row (`Projectile.cs:5404-5412` — 26×26,
  aiStyle 9, friendly, melee, pen −1) + `AI_009_MagicMissiles`'s
  type-491 head (`:43197-43200` — the `_Old` dispatch + return) + the
  `AI_009_MagicMissiles_Old` body (`:43427-43653`) ride ONE wave; its
  5 dust sites (`:43574/43593/43610/43627/43635`) activate by the e3
  emitter pattern in that wave's dust follow-up — NOTE only the
  `:43572-43582` half (the Next(2) gate + the Next(3) type switch
  15/57/58, alpha 255 scale 0.7, vel×0.25, position=(pos+base.pos)/2)
  is 491-reachable: the `:43593/43610/43627/43635` sites sit under
  `type == 79/16/34` gates INSIDE `_Old` — dead for the only reachable
  kind 491, kept verbatim when ported (the census-F2 convention; the
  `:43434` type-16 speed branch likewise). The 491-specific machine
  halves the wave must keep: the HandleMovement `ai[0] = −10` + the
  ×−1 bounce (`:18464-18479`), the <100f close-range ×1.1 boost +
  normalize-cap-20 (`:43452-43467`), the (v×4+aim)/5 blend
  (`:43481-43485`), the localAI[0]>15 return-flight + <20f kill +
  the 25-speed homing (`:43540-43553`), the >2000f owner-distance
  kill (`:43568-43571`), the GetAlpha White (`:79204-79207`).

### (9) The g63 doc-66 correction — the mutual receipt CLOSED

The e3 → g63 request (§3 above) is **CONSUMED + FIXED on g63's branch**:
g63's RETIRE commit `1c52d5ee3` (research/240 FINAL, this seal's fetch)
corrected `docs/research/66-proj-trail-dust-claims.md:79` to
`| 60 | 358 (:29167/29191 ONLY …) |` with the full attribution note
(`:29205/29229` → the 406's/g64's, `:29245/29284` → the 1042's/e3
`style60_shimmer_gun_trail_1042`'s, the `:29234` `type != 1042` return
cited) — re-read verbatim at the tip this round. The e3 correction
(`:29245/29284` = the 1042's, NOT the 358's) and the g63 fix AGREE
line-for-line; no code collision ever existed (g63's emitter cites
`:29158-29196` only). **The reciprocal receipt**: g63's research/240
§3.2 flagged the THIRD copy of the 12/955 anniversary re-pick error —
e3's `style5_star_dust` drew ONE Next(4) where the C# `:23975-23978`
draws TWO (the second SelectRandom wins). **FIXED on this branch this
round** (the emitter + its mirror witness take the two-draw form; the
g63 `036901a5e` / g16 `falling_star_trail` pin). Both directions of the
mutual receipt are now closed; the merged 12/955 arm keeps ONE consumer
of the shared `:23973-23981` gore draws (g63 research/199's 12 row).

### (10) The AI-inline-family bible — FINAL (the three-disjoint-KEEP)

The family declaration, agreed across the three lanes' FINAL docs
(g63 research/240 §1-2, g64 research/196 FINAL union-table, this doc):

| axis | lane | file | scope | tip at this seal |
| ---- | ---- | ---- | ----- | ---------------- |
| per-kind ≤400 TRAIL | W4-g63 | `projectile_trail_dust.rs` | the rowed kinds' arms + the unrowed emitters (research/199 activation checklist; doc-66 keeps 66) | `1c52d5ee3` (RETIRE FINAL 240) |
| per-kind ≥401 TRAIL | W4-g64 | `proj_trail_dust.rs` + the remainder machines | 24 WIRED kinds + 21 cited-unrowed + the 2 spark riders (research/196) | `3e581c8ce` (SEAL-VERIFY; the GoreRng→DustDraw lift executed, the f50 BlastRng deletion is the merge instruction) |
| aiStyle-generic + the site windows | **W4-e3 (this)** | `projectile_dust.rs` | the `:22297-24318` generic window (styles 2/4/5/6/8/152) + the AI-INLINE-40 (34 STAGED + 3 WIRED) + the AI_001 edges 2 + the `:78227-78508` Kill_Bombs tail 15 | this seal |

**Three DISJOINT axes, all KEEP at merge** (the d54 bible §8.3 step 9).
The kind gates ≤400/≥401 never overlap; the e3 axis is aiStyle-generic +
the cited site windows — no body overlap with either per-kind axis. The
only retirements are the per-kind twins that feed the surviving
machines: 497/504/503 → e3 `ai_style2`/`ai_falling_star_family` (g64
FINAL 2 executed the 503 constants-copy), 723/1133 → f48 `ai_cloak_star`,
596 → d11 `ai_nebula_sphere` 575 + the g64 bool seam, 813/895/898/957/650
→ the g64 dust-wired machines (the d13 row supersets kept). The
`clone_dust` genericization keep-one is CONFIRMED BOTH WAYS (§4 + g64's
RETIRE (1) + its SEAL-VERIFY (2)). The Kill-family death sites stay
g65's (`f6e07bb90` — the Kill-remainder-234 census CLOSED, 0 unclaimed);
the Orch-61 stays d54/e4's (§5). NO site in the union is unclaimed.

**The e3 lane is SEALED.** This round's deltas: the 955 anniversary
two-draw fix (the g63 §3.2 reciprocal), the 125→231 registry rename, the
FINAL activation-order table (7), the 981/491 task seeds closed (8), the
g63 mutual receipt closed (9), this bible (10). No further site claims;
later tip movement is recorded as receipts, not re-openings. Code-only
throughout — the fix's witness is WRITTEN NOT RUN; all evidence is
git-tip reads + C# line reads (the `:23975-23978` re-pick re-read
verbatim this round).
