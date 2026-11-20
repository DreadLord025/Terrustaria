# 65 — World-sim dust-site batches (W4-d54 / W4-e4 / W4-f6): claims + census

Status: CLAIM doc (W4-f6, batch 3 of the world lane). Written early per the
lane's coordination rule — the branch list is the claim record; this doc pins
the file split so the world batches never overlap the entity lanes or the
g11-g15 Main/WorldGen/WorldItem lanes.

## Lane boundary (what "world" means here)

The dust-site waves split by C# caller file:

- **Player.cs** — d51 / e1 / f3 / g66 (player lanes).
- **NPC.cs** — d52 / e2 / f4 / g59-g62 (NPC lanes).
- **Projectile.cs** — d53 / e3 / f5 / g63-g65 (projectile lanes).
- **Main.cs (23 sites)** — g11 / g12 / g13, per the pushed
  `64-dust-main-sites-claims.md` thirds split. CLAIMED, untouched here.
- **WorldGen.cs (29 sites)** — g14 (`task/W4-g14-dust-worldgen-sites`).
  CLAIMED by branch, untouched here.
- **WorldItem.cs (17 sites)** — g15 (`task/W4-g15-dust-item-sites`).
  CLAIMED by branch, untouched here.
- **TileDrawing.cs (53)** — d56-d60 tile-draw lanes. Untouched here.

The **world lane** (d54 first / e4 second / f6 third) is everything else that
spawns dust from the world simulation: the mount machine, the tile-edit /
chest / wiring helpers, the liquid renderer, the floating-text / pylon /
void-lens helpers, and the net / rain riders. d54 and e4 pushed **zero**
commits (dead lanes), so every site below is unclaimed; f6 takes the next
~40 in line order, 10 per commit.

## Census (f6's ~40, line-ordered per file)

`Mount.cs` — **27** sites (the minecart-213 wheel sparks are already
`minecart.rs`; these are the OTHER mount dusts):

| Line | Family | Enclosing method |
| ---- | ------ | ---------------- |
| 3113 | NewDust | `UseDrill` (`:3059`) block beam |
| 3170 | NewDust | `UseDrill` wall beam |
| 3878 | NewDust | `UpdateFrame` (`:3581`) case 10/47 dash ring |
| 3896 | NewDust | `UpdateFrame` case 10 dash body |
| 3939 | NewDust | `UpdateFrame` case 44 dash ring |
| 3970 | NewDust | `UpdateFrame` case 44 run puff |
| 4000 | NewDust | `UpdateFrame` case 45 dash ring |
| 4023 | NewDust | `UpdateFrame` case 45 dash body |
| 4103 | NewDust | `UpdateFrame` case 14 basilisk run |
| 4123 | NewDust | `UpdateFrame` case 14 basilisk charge |
| 4626 | NewDustDirect | `EmitGolfCartSmoke` (`:4623`) |
| 4703 | NewDustPerfect | `DoGemMinecartEffect` (`:4697`) |
| 4719 | NewDustPerfect | `DoSteamMinecartEffect` (`:4712`) |
| 4750 | NewDustPerfect | `DoExhaustMinecartEffect` (`:4729`) |
| 4782 | NewDustPerfect | `DoConfettiMinecartEffect` (`:4763`) |
| 4824 | NewDustPerfect | `UpdateEffects` (`:4795`) case 23 broom sparkle |
| 4849 | NewDustPerfect | `UpdateEffects` case 23 broom trail |
| 5061 | NewDust | `UpdateEffects` case 11 drill-mount trail (lerp) |
| 5071 | NewDust | `UpdateEffects` case 11 drill-mount trail (single) |
| 5099 | NewDustPerfect | `UpdateEffects` case 22 lava-cart trail |
| 5149 | NewDustDirect | `UpdateEffects` case 61 pixie tail |
| 6339 | NewDust | `DoFailedDismountDust` (`:6320`) |
| 6382 | NewDustDirect | `DoSpawnDust` (`:6371`) wolf 52 |
| 6391 | CloneDust | `DoSpawnDust` wolf 52 clone |
| 6404 | NewDustDirect | `DoSpawnDust` chillet 62-65 |
| 6413 | CloneDust | `DoSpawnDust` chillet clone |
| 6493 | NewDust | `DoSpawnDust` generic tail |

Small world files — **13** sites:

| File:Line | Family | Enclosing method |
| --------- | ------ | ---------------- |
| `Chest.cs:420` | NewDust | `Unlock` (`:346`) 4-per-tile |
| `Utils.cs:1089` | NewDust | `PoofOfSmoke` (`:1078`) 10-dust tail |
| `GameContent/TeleportPylonsSystem.cs:390` | NewDust | `SpawnInWorldDust` (`:327`) |
| `GameContent/VoidLensHelper.cs:48` | NewDustDirect | `SpawnVoidLensDust` (`:40`) arm A |
| `GameContent/VoidLensHelper.cs:63` | NewDustDirect | `SpawnVoidLensDust` arm B |
| `PopupText.cs:461` | NewDustPerfect | `EmitFancyFlashDust` (`:442`) loop 1 |
| `PopupText.cs:470` | CloneDust | `EmitFancyFlashDust` loop 1 clone |
| `PopupText.cs:479` | NewDustPerfect | `EmitFancyFlashDust` loop 2 |
| `PopupText.cs:488` | CloneDust | `EmitFancyFlashDust` loop 2 clone |
| `GameContent/Liquid/LiquidRenderer.cs:560` | NewDust | `InternalPrepareDraw` (`:161`) lava bubble |
| `GameContent/Liquid/LiquidRenderer.cs:564` | NewDust | `InternalPrepareDraw` lava bubble big |
| `Rain.cs:122` | NewDust | `Rain.Update` (`:96`) splash |
| `MessageBuffer.cs:3697` | NewDust | `GetData` case 102 nebula level-up |

**Already ported / dead (not re-ported):** `Collision.cs:3403` (the cobweb
puff rides `collision.rs::sticky_tiles`' `web_puff` → `player.rs`
`pending_sticky_dust`), `Collision.cs:3427` (dead — `type == 51` inside the
honey-block else-arm, never true), `WaterfallManager.cs:1061`
(`waterfall/draw.rs::try_sparkling` → `rain.rs` sparkle pool).

## Batch split (10 sites per commit)

- **f6 batch 1** — Mount movement: 3113/3170 (UseDrill) + 3878/3896/3939/
  3970/4000/4023/4103/4123 (UpdateFrame dash/gallop). 10 sites.
- **f6 batch 2** — Mount effects: 4626/4703/4719/4750/4782/4824/4849/5061/
  5071/5099 (EmitGolfCartSmoke + the minecart effects + UpdateEffects
  broom/drill/lava). 10 sites.
- **f6 batch 3** — Mount spawn/dismount: 5149/6339/6382/6391/6404/6413/6493
  (pixie tail + DoFailedDismountDust + DoSpawnDust). 7 sites.
- **f6 batch 4** — small world files: Chest 420, Utils 1089, Pylons 390,
  VoidLens 48/63, PopupText 461/470/479/488, LiquidRenderer 560/564, Rain
  122, MessageBuffer 3697. 13 sites.

## Contract

Pool: `origin/task/W4-d50-dust-type-table`
(`tr_core::particles::DustPool::new_dust` / `new_dust_direct` /
`new_dust_perfect` / `clone_dust`, `Dust.cs:75-286` verbatim).

RNG: one slot per C# pass — each ported site drains on the stream of the
`Main.SwapRandom` pass that executes it in the C# (never a new stream,
S1-A-10). The mount / chest / poof sites ride `UpdatePlayers`
(`Player.Update`, `Main.cs:17754`); the pylon `SpawnInWorldDust` rides the
caller's pass (`Main.TeleportEffect` = g11/g12's, `Player.UpdateTeleportVisuals`
= UpdatePlayers); the void-lens rides `UpdateProjectiles`
(`Projectile.cs:52195`); the floating-text rides `UpdateFloatingText`
(`Main.cs:17791`); the lava bubbles ride the renderer's own `_random`
(`LiquidRenderer.cs:132`, a private stream — documented); the rain splash
rides the draw's `Main.rand` (`Main.DrawItems` `:38821`, no swap — the bare
global); the nebula level-up rides the net dispatch's pass.

Code-only mode: WRITTEN NOT RUN, no cargo/rustc/rustfmt/tests/graphify
invoked (user directive). The witnesses are owed a compile window before
merge. Emitters whose enclosing machine is unported are staged (the e2
Skeletron/Brain precedent) with draw-mirror witnesses; the live ones (Chest
Unlock, Utils.PoofOfSmoke, the mount arms present in `mount.rs` /
`player.rs::update_mount`) are wired.
