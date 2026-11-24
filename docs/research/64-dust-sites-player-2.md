# W4-e1 — Player.cs Dust.NewDust sites, SECOND batch (claim ledger)

Task: `task/W4-e1-dust-sites-player-2`. Continues W4-d51
(`task/W4-d51-dust-sites-player`, tip `0d0cf7c4`), which landed 26 sites in
three lanes: extra-jump chain 20820–21009, DashMovement 21265–21502,
fast-run 20219–20294 (+ boot-run visual grants). DustPool contract: W4-d50
(`task/W4-d50-dust-type-table`, tip `2e243a2d`, `dust_physics.rs`).

## Census

`decompiled/Terraria/Player.cs` carries **215** `Dust.NewDust*`-family call
sites (research/15 caller table). Minus d51's 26, minus the main-ported
recall/mirror family (42980/42990/42996/43115/43123/43133 —
`recall_teleport.rs`, S1-P-i38) and StickyMovement's six (23036–23076 —
`world_sim/player.rs` sticky drain, main), the unclaimed pool is ~183.
This task takes the **next 40 in line order** whose hosts exist (or land
here as minimal grants) on `main` `a725acb0`.

## Claimed sites (40, line order)

| # | C# line | Host block | Batch |
|---|---------|-----------|-------|
| 1 | 12569 | `Yoraiz0rEye` trail (`:12446-12577`, slot grant `:36969-36972`, tick gate `:25487`) | 1 |
| 2 | 14291 | `DoGlassSlipperSparkles` idle sparkle (`:14286-14306`, 5077 grant) | 1 |
| 3 | 14300 | `DoGlassSlipperSparkles` walking sparkle (same) | 1 |
| 4 | 16389 | `ApplySetBonus_Solar` 16-shield burst (`:16366-16402`, shield counter grant) | 1 |
| 5 | 16623 | `UpdateTeleportVisuals` style-0 (`:16613-16670`, teleportTime/Style grant) | 1 |
| 6 | 16633 | `UpdateTeleportVisuals` style-1 (same) | 1 |
| 7 | 16652 | `UpdateTeleportVisuals` style-4 portal (same) | 1 |
| 8 | 19273 | `UpdateLifeRegen` shiny-stone sparkle (`:19268-19284`, 3337 grant) | 1 |
| 9 | 19296 | `UpdateLifeRegen` crimson heal burst (`:19292-19305`) | 1 |
| 10 | 19555 | `UpdateManaRegen` full-bar burst (`:19550-19560`) | 1 |
| 11 | 20320 | `MowTheLawn` swing puff (`:20301-20346`, held 4049) | 2 |
| 12 | 21654 | `WallClimbMovement` climb-up puff (`:21607-21748`, mount-55 host port) | 2 |
| 13 | 21678 | `WallClimbMovement` climb-down puff (same) | 2 |
| 14 | 21729 | `WallClimbMovement` slide puff (same) | 2 |
| 15 | 21790 | `WallslideMovement` down puff (`:21750-21840`, host port) | 2 |
| 16 | 21826 | `WallslideMovement` slide puff (same) | 2 |
| 17 | 21901 | `DoubleJumpVisuals` Cloud (`:21892-22021`, performing-jump lifecycle) | 2 |
| 18 | 21915 | `DoubleJumpVisuals` Sandstorm ×3 (same; gore `:21922` = d55 skip) | 2 |
| 19 | 21934 | `DoubleJumpVisuals` Fart (same) | 2 |
| 20 | 21941 | `DoubleJumpVisuals` Unicorn (same) | 2 |
| 21 | 21962 | `DoubleJumpVisuals` Sail ×1/2 (same) | 3 |
| 22 | 21989 | `DoubleJumpVisuals` Blizzard ×2 (same) | 3 |
| 23 | 22005 | `DoubleJumpVisuals` Blizzard +0.8 ×3 (same) | 3 |
| 24 | 22014 | `DoubleJumpVisuals` Blizzard −0.8 ×3 (same) | 3 |
| 25 | 23316 | `CheckDrowning` bubble (`:23283-23316`) | 3 |
| 26 | 23983 | `MakeFloorDust` footstep/landing (`:23790-24043`, FloorVisuals seam) | 3 |
| 27 | 24274 | `UpdateHairDyeDust` team-dye puff (`:24268-24295`; gore `:24285` = d55 skip) | 3 |
| 28 | 24641 | `manaHeat` ambient (`Player.Update :24630-24662`, CloneDust 24654) | 3 |
| 29 | 24654 | `manaHeat` CloneDust sibling (same block, counted as its own site) | 3 |
| 30 | 25228 | stoned-transition burst (`Player.Update :25219-25234`) | 3 |
| 31 | 25877 | Vortex stealth 229 ring (`:25872-25893`, setVortex grant) | 4 |
| 32 | 25887 | Vortex stealth 240 ring (same) | 4 |
| 33 | 27785 | liquid ENTRY shimmer burst (`:27781-27806`) | 4 |
| 34 | 27812 | liquid ENTRY honey burst (`:27808-27819`) | 4 |
| 35 | 27825 | liquid ENTRY water burst (`:27821-27832`) | 4 |
| 36 | 27839 | liquid ENTRY lava burst (`:27835-27846`) | 4 |
| 37 | 27879 | liquid EXIT shimmer burst (`:27875-27900`) | 4 |
| 38 | 27906 | liquid EXIT honey burst (`:27902-27913`) | 4 |
| 39 | 27919 | liquid EXIT water burst (`:27915-27926`) | 4 |
| 40 | 27933 | liquid EXIT lava burst (`:27929-27940`) | 4 |

Note: site 29 (24654) is a `Dust.CloneDust` — the family census counts it
(research/15 counts CloneDust 51 tree-wide). If the merger prefers strict
`NewDust*`-only counting, the 40th slot is 27933 either way; the ledger
lists both.

## Avoided (other lanes / unclaimed follow-ups)

- **g66 player trails** (`task/W4-g66-player-trail-dust`, based on d51 tip):
  the wing-flight trail 27047–27304, WingFrame 29403–30512, WingAirVisuals
  30579–31006 — NOT touched here.
- **d115/d116** (dodge family / hurt armor, in flight): dodge bursts
  38062/38073/38124 + Hurt blood 38536–38550 — NOT touched.
- **g15 dust item sites**: the item-use visuals 43729/43746, 44758–45089,
  49803–51341 — NOT touched.
- Remaining unclaimed after this batch (line order): 29403+ (g66),
  31727 (shield parry — no host), 36005–36193 (`SetArmorEffectVisuals` —
  no head/body/legs slot fields), 38873 (anniversary death — no
  tenthAnniversary player field), 39038–39053 (`KillMe_DustExplosion` —
  frost/bone armor flags unported), 43242 (rod of discord — item lane),
  56850/56860/57020 (minion rest visuals).

## Convention

Same as d51/e2: the tick snapshots WHERE a burst goes at the C# read
moment; `player_dust2.rs` replays each `Dust.NewDust` on the shared
`UpdatePlayers` stream (`Main.SwapRandom`, `Main.cs:17755-17757`) in exact
C# statement order (argument draws FIRST, pool internals inside
`DustPool::new_dust`, post-writes after — the sentinel slot 6000 takes the
writes exactly like the C#). Gore siblings stay W4-d55's (argument draws
consumed in order where they interleave). `gravDir` folds to the vanilla
`1f` (no gravity flip in the port). `shader`/`customData` are dropped
fields (the pool contract). Code-only mode: witnesses WRITTEN NOT RUN.

## Completion (2026-11-21)

All 40 sites landed on `task/W4-e1-dust-sites-player-2`:

| Commit | Scope |
|--------|-------|
| `7c4b7d15` | claim: census + this ledger |
| `b1a96d0c` | batch 1 (sites 1–10, lines 12569–19555) |
| `e8219a01` | batch 2 (sites 11–20, lines 20320–21941) |
| `73d0aa6b` | batch 3 (sites 21–30, lines 21962–25228) |
| `53add802` | batch 4 (sites 31–40, lines 25877–27933) |
| `3245d624` | drain execution-order + fidelity fixes below |
| `164596b8` | e1b reconcile: the f3-reconcile WIP completed + 2 drain-order fixes (research/141) |

26 draw-mirror witnesses in `player_dust2.rs` + 15 `w4_e1b_*` host
witnesses in `player.rs` (the f3-reconcile folds) — WRITTEN NOT RUN
(code-only mode; no cargo/graphify runs this task).

### The e1b reconcile (f3 `75f3f734` MOVE-RETIRE verdicts, research/141)

- **Wall copy — DROP at the merge, five deviations FIXED here**: the
  vacuous gravDir-1 clauses dropped from the climb gate (`:21639`), the
  level-1 slide gate (`:21709`) and the wallslide gate (`:21777`); the
  unguarded `vy < -0.5` single-boot slide (`:21814`) removed; the climb
  probe uses `(int)v >> 4` (`Utils.cs:1807-1810`), not `/16`. The block
  gained the `!grappling && !tongued` guard (`:26494`).
- **Stealth — spawn 1.0** (`:1562`, was 0.0 WRONG), the final-else
  `stealth = 1f` (`:25896-25899`) landed, the hurt reveal (`:38245`,
  keep-one vs f50), ResetEffects `aggro = 0` (`:18377`), the
  climbing/frog grants (`:15038-15066`/`:14426-14444`), the Vortex
  ranged kb (`:53364-53367`), the Cloud/Sandstorm suppression
  (`:21894`), the dash stealth cancel (`:21379`).
- **Floor dust — KEEP-E1**: the falling head capture (`:28073-28081`)
  fires the landing burst (f3's rest-path capture never does); the
  any-wet/cart gate (`:23740`), the Stinky buff-120 grant (`:23732`)
  and the integer half (`:24026`) re-verified.
- **Drown bubble** — the host gates `flag` only; the wet flags ride the
  snapshot, the Next(20) draws BEFORE the wet refuse (`:23283` || order).
- **Drain order** — MowTheLawn (`:20178`) now drains BEFORE the jump
  sounds (`:26744`); the drown gasp (`:23257`) inside CheckDrowning
  before the bubble.

### Fidelity fixes after the four batches (re-read of the C#)

- **Drain order = execution order, not line order.** `UpdateHairDyeDust`
  (`:24593`) and the manaHeat block (`:24630`) run BEFORE the stoned
  transition (`:25219`), the Vortex rings (`:25872`, UpdateEquips tail
  `:25633`), CheckDrowning (`:27655`), the splash block (`:27772`) and
  FloorVisuals (`:28271`). The `world_sim/player.rs` drain was reordered to
  match; the floor dust now drains after the splash-sound drain.
- **`falling` is a movement-head capture** (`:28073-28081`, |vy| > gravity
  BEFORE the collision resolve) — a landing tick reaches FloorVisuals with
  vy==0 but falling=true, so the 20-dust landing burst fires. Batch 3's
  initial hard-fold to `false` was wrong; fixed.
- **Stoned burst fires on EITHER edge** (`:25219` `stoned != lastStoned`);
  only the 20-damage Hurt is rising-edge-gated (`:25221`). Fixed.
- **Hair Dye Remover (1990)** keeps `hairDye = 0` (`Item.cs:48714` skips the
  shader-index post-processing when the raw value is 0); the use grant
  special-cases it instead of storing −1.
- **Losing the Vortex set disarms the toggle**
  (`UpdateArmorSets_Always_Vortex`, `:16315-16321`).

### Documented deviations (per-site folds)

- **23316**: the breathing-reed (`hasBreathingReed`) and mount-mouth
  (`MouthPosition`, mounts 52/54) anchor adjustments have no port fields —
  the base anchor (direction×12, +4) stands; `gravDir` folds to 1.
- **23983**: the shimmer-block ParticleOrchestrator arm (`:23792-23818`,
  tiles 659/667) is the particle-orchestrator lane's — its draws are NOT
  NewDust and stay there. The 666/712 landing sound (Item177) is the sound
  lane's; the FloorVisuals buff-120 grant (`:23732`) LANDED with the e1b
  reconcile (the g66 row). The TryBouncingBlocks FloorVisuals caller
  (`:23567`/`:28223`) is the bounce lane's (unported) — only the grounded
  caller (`:28271`) fires here.
- **24274**: the gore sibling (`:24285`) body is W4-d55's; its Next(225)
  gate + argument draws + post-write draws are consumed here in order.
  `hairDye` stores the shader index (`TeamDyeShaderIndex` = 9 = Party Hair
  Dye 1984's bind, `ID/ContentSamples.cs` DyeShaderIDs.Initialize); the use
  grant lives in `world_sim/editing.rs` (press-edge stands in for
  ItemTimeIsZero).
- **24641/24654**: latent — `mana_heat` stays 1.0 in the port (no writer;
  the buff-396 decay `:10328-10341` is unported), but the machinery is
  ported faithfully (`Utils.Remap` clamped, NewDustDirect 267 + CloneDust).
- **25877/25887**: the stealth-chain's final else (`:25896-25899`,
  `stealth = 1` without any stealth armor) NOW LANDS in the no-set
  fallback (the e1b reconcile; the spawn default is 1.0 per `:1562`).
  The Psycho-Knife/Shroomite arms are f3's machine — at the e1b-141
  merge e1's vortex-arm-only machine + rings drop to f3's full chain
  (research/141). The ±0.04 ramp, ranged damage/crit/aggro grants,
  run-speed ×0.3 and the mount cancel are ported.
- **27785–27933**: the exit jump clamp (`:27864`) is a movement seam; the
  splash gate stands alone (the S1-E-21f sound block's convention). Dust
  drains BEFORE the splash sound (C# dust-then-PlaySound within the block).
