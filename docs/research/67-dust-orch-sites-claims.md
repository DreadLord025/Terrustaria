# 67 — ParticleOrchestrator dust sites FIRST batch (W4-d54 batch 3+): claims + census

Status: CLAIM doc (W4-d54, the world lane's orchestrator batches). Written
after the in-flight probe (2026-11-21, `git fetch origin --prune` + branch
scan). The branch list is the live claim record; this doc pins the site
split so the orchestrator batches never overlap e4/f6 or the entity lanes.

## In-flight probe (2026-11-21)

| Lane | Claim state | Verdict |
| ---- | ----------- | ------- |
| WorldGen.cs (29 sites) | `origin/task/W4-g14-dust-worldgen-sites` **a35efec2 LANDED all 29** (`worldgen_dust.rs` + `worldgen_dust_wall.rs` + `worldgen_dust_tile.rs`) | COVERED — d54's own batch 1-2 WorldGen bodies (414919c2/e5a7c743) overlap g14's; the merger takes g14's tr-core versions and re-points the editing.rs seams (noted below) |
| Main.cs (23 sites) | g11/g12/g13 branches pushed; g12's census `64-dust-main-sites-claims.md` pins all 23 in line-order thirds (g11 1-8, g12 9-16, g13 17-23) | ALL CLAIMED — d54 takes NO Main.cs site |
| WorldItem.cs (17) | g15 branch pushed | CLAIMED |
| ParticleOrchestrator.cs (61) | e4's claim `66-dust-world-sites-2-claims.md` **reserves sites 1-40 for d54** ("ArmletOfRuinRecharge through CopperSlimeEffect", lines 423-2582); e4 takes 41-61 + Mount 1-19 (claim only, no code) | d54's lane = orch 1-40 |
| Mount.cs + 13 small files | f6's claim (`65-dust-world-sites-claims.md` on `task/W4-f6-dust-world-3`, 960fc28d) — claim only, no code; f6's "d54/e4 dead-lane" note predates d54's batch-1/2 push and is superseded by e4's reservation | untouched here |
| Wiring.cs / NetMessage.cs / TileObject.cs | direct-path census: **zero** `Dust.NewDust*` sites (the net rider is `MessageBuffer.cs:3697`, f6's) | nothing to port |

## Census (orch sites 1-40, C# line order — the e4-reserved lane)

Counting convention = e4's: `NewDust` + `NewDustPerfect` + `NewDustDirect`;
`CloneDust` siblings ride their parent's ledger entry (they re-roll a fresh
`NewDust` inside, `Dust.cs:269`).

| # | Line | Family | Enclosing method |
| - | ---- | ------ | ---------------- |
| 1 | 423 | NewDust | `Spawn_ArmletOfRuinRecharge` (`:418`) ×10 |
| 2 | 442 | NewDust | `Spawn_PyroclasticStoneRecharge` (`:437`) ×15 |
| 3 | 459 | NewDust | `Spawn_SnappingStoneRecharge` (`:454`) ×10 |
| 4 | 628 | NewDustDirect | `Spawn_SnappingStone` (`:593`) ×20 |
| 5 | 723 | NewDustPerfect | `Spawn_SnappingStoneMelee` (`:648`) pair ×2 |
| 6 | 726 | NewDustPerfect | `Spawn_SnappingStoneMelee` pair ×2 |
| 7 | 791 | NewDustPerfect | `Spawn_SnappingStoneSummon` (`:733`) ×15 |
| 8 | 804 | NewDustPerfect | `Spawn_SnappingStoneSummon` ×8 |
| 9 | 826 | NewDust | `Spawn_PaladinsHammerShockwave` (`:815`) ×50 gate |
| 10 | 835 | NewDust | `Spawn_PaladinsHammerShockwave` ×50 gate |
| 11 | 932 | NewDustDirect | `Spawn_PaladinsShieldHit` (`:902`) ×4 |
| 12 | 964 | NewDustDirect | `Spawn_PaladinsShieldHit` ×4 |
| 13 | 1084 | NewDustDirect | `Spawn_StormLightning` (`:1026`) tree-6 ×6 |
| 14 | 1091 | NewDustDirect | `Spawn_StormLightning` tree-31 ×3 |
| 15 | 1168 | NewDustPerfect | `SpawnLightningExplosionDust` (`:1164`) ×12 |
| 16 | 1181 | NewDustPerfect | `SpawnLightningExplosionDust` ×6 |
| 17 | 1196 | NewDustPerfect | `SpawnLightningExplosionDust` ×6 |
| 18 | 1209 | NewDustPerfect | `SpawnLightningExplosionDust` ×6 (+clone :1214) |
| 19 | 1220 | NewDustPerfect | `SpawnLightningExplosionDust` ×6 (+clone :1225) |
| 20 | 1231 | NewDustPerfect | `SpawnLightningExplosionDust` ×3 |
| 21 | 1446 | NewDust | `Spawn_RainbowBoulder4` (`:1431`) ×14/8 |
| 22 | 1545 | NewDustDirect | `Spawn_RainbowBoulder1` (`:1502`) ×21 |
| 23 | 1628 | NewDust | `Spawn_RainbowBoulder3` (`:1585`) ×4 (+clone :1638) |
| 24 | 1693 | NewDustPerfect | `Spawn_MoonLordWhip` (`:1648`) pair ×2 |
| 25 | 1697 | NewDustPerfect | `Spawn_MoonLordWhip` pair ×2 |
| 26 | 1775 | NewDustPerfect | `Spawn_DeadCellsBarrelExplosion` (`:1705`) ×7 |
| 27 | 1886 | NewDustPerfect | `Spawn_DeadCellsFlint` (`:1781`) ×30 |
| 28 | 2018 | NewDust | `SpawnHelper_SpawnSingleLineDust` (`:2016`) per line step |
| 29 | 2088 | NewDustPerfect | `Spawn_ShadowOrbExplosion` (`:2028`) pair ×2 |
| 30 | 2091 | NewDustPerfect | `Spawn_ShadowOrbExplosion` pair ×2 |
| 31 | 2144 | NewDustPerfect | `Spawn_DeadCellsDownDashExplosion` (`:2120`) ×40 + inline clone |
| 32 | 2293 | NewDust | `Spawn_ShimmerTownNPC` (`:2254`) ×20 |
| 33 | 2335 | NewDust | `Spawn_Digestion` (`:2328`) ×3+Next(4) |
| 34 | 2384 | NewDustPerfect | `Spawn_LoadOutChange` (`:2370`) ×40 |
| 35 | 2392 | NewDustPerfect | `Spawn_LoadOutChange` ×5 |
| 36 | 2530 | NewDustPerfect | `Spawn_DeadCellsBarnacleShotFiring` (`:2521`) ×15 |
| 37 | 2541 | NewDustPerfect | `Spawn_DeadCellsBarnacleShotFiring` ×5 |
| 38 | 2555 | NewDustPerfect | `ElderSlimeEffect` (`:2551`) ×30 |
| 39 | 2574 | NewDustPerfect | `NerdySlimeEffect` (`:2569`) ×60 |
| 40 | 2582 | NewDustPerfect | `CopperSlimeEffect` (`:2578`) ×40 |

40 ledger entries — e4's count exactly (16 NewDust + 37 NewDustPerfect +
8 NewDustDirect across the whole file; the window 423-2582 carries
10 + 24 + 6). e4's boundary "2582 is d54's, 2653 is e4's" holds.

Dust-free orch functions inside the window (NOT sites; RNG-bearing ones
still consume their argument draws when their parent emitter runs):
`Spawn_HarpyCharm` :472 (Fading/Sparkle only), `Spawn_InScreenDungeonSpawn`
:881 (gore only — g17's lane), `Spawn_ArcLightning` :1241, `Spawn_NatureFly`
:1319, `Spawn_LakeSparkle` :1332, `Spawn_FakeFish` :1351,
`Spawn_RainbowBoulder2` :1558, `Spawn_MoonLordWhipEye` :1891, the
UFOLaser/MagnetSphereBolt/HeatRay/Shadowbeam wrappers :1950-1994 (they
route through `SpawnHelper_SpawnInLine` :1996 → site 28), `Spawn_PooFly`
:2317, `Spawn_ShimmerBlock` :2349, `Spawn_TownSlimeTransform` :2401 (the
dispatch to 38/39/40), `Spawn_DeadCellsMushroomBoiExplosion` :2417 (gore/
particle only), `Spawn_DeadCellsMushroomBoiTargetFound` :2506.

## Batch split (10 sites per commit; d54's batches 1-4 already landed the
original WorldGen/Main census, so the orchestrator lane starts at batch 5)

- **Batch 5 (sites 1-10)**: recharge trio + SnappingStone trio +
  PaladinsHammerShockwave pair. `orch_dust.rs`.
- **Batch 6 (sites 11-20)**: PaladinsShieldHit pair + StormLightning tree
  pair + SpawnLightningExplosionDust's six arms. `orch_dust_lightning.rs`.
- **Batch 7 (sites 21-30)**: rainbow boulders + MoonLordWhip pair +
  DeadCellsBarrel/Flint + the line-dust helper + ShadowOrb pair.
  `orch_dust_boulder.rs`.
- **Batch 8 (sites 31-40)**: DownDash + ShimmerTownNPC + Digestion +
  LoadOutChange pair + BarnacleShot pair + slime trio.
  `orch_dust_town.rs`.

## Contracts

- **Pool**: `tr_core::particles::DustPool::new_dust` / `new_dust_direct` /
  `new_dust_perfect` / `clone_dust` (d50, `Dust.cs:75-286` verbatim).
  Post-spawn writes land on `slot_mut(idx)` — the sentinel takes them like
  `Main.dust[6000]`.
- **RNG**: one slot per C# pass. The orchestrator spawns run synchronously
  inside `RequestParticleSpawn`/`BroadcastParticleSpawn`
  (`ParticleOrchestrator.cs:52-90` → `SpawnParticlesDirect` :162), so each
  emitter drains on the stream of the pass that invoked it — the caller
  passes `&mut UnifiedRandom`; never a new stream (S1-A-10). Argument
  draws run BEFORE the pool internals (C# evaluates arguments first).
- **Particle-pool requests** (`FadingParticle`/`PrettySparkle`/
  `RandomizedFrame`/`Flame`) are the documented renderer skip (research/15:
  no particle renderer); `ParticlePool.RequestParticle` is RNG-free
  (`Graphics/Renderers/ParticlePool.cs:32`) but their ARGUMENT draws are
  consumed in order — the port keeps them as ordered `next_*` calls with
  the visual writes dropped (each site cites the dropped range).
- **Gore siblings** (PaladinsHammerShockwave :871, StormLightning :1102,
  DeadCellsFlint :1850, Digestion :2341, ElderSlime :2565,
  ShadowOrbExplosion :2099's dead `j < 0` loop) are g16/g17's lane; their
  argument draws are consumed here in order (the e2/e4 split).
- **KillTile_MakeTileDust delegation**: the tile sweeps (sites 9/10 and
  batch 4/5's StormLightning/DeadCellsFlint) call this branch's own
  `world_dust_tile::make_tile_dust` + `tile_dust_amount` (batch 3,
  886764c4) — the same function g14 landed in tr-core; the merger keeps
  one copy and re-points the call.
- **Overlap note (merger)**: d54 batches 1-4 (414919c2/e5a7c743/886764c4/
  48343a4c) ported the ORIGINAL 40-site census (WorldGen 29 + Main first
  11) into `tr-app/world_dust*.rs` while g14 landed all 29 WorldGen sites
  in tr-core and g11-g13 claimed all 23 Main sites. g14's tr-core versions
  win for WorldGen; the Main first-11 bodies overlap g11's sites 1-8 +
  g12's 9-11 — the merger reconciles per the thirds split (g11/g12's
  verbatim bodies win where both exist; d54's editing.rs live seams stay).
  Recorded here so the merge reconciles instead of double-spawning. The
  orchestrator batches 3-6 below are d54's UNCLAIMED lane per e4's
  reservation — no overlap.

WRITTEN NOT RUN (code-only mode): draw-mirror witnesses per batch, owed a
compile window before merge.
