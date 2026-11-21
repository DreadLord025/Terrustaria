# 66 — World dust sites SECOND batch (W4-e4): claims + census

Status: CLAIM doc (W4-e4, the world lane's second batch). Written early per
the lane's coordination rule — the branch list is the live claim record;
this doc pins the census + the line-order split so the batches never
overlap.

## In-flight probe (2026-11-20, `git fetch origin --prune` + branch scan)

| Lane | Claim state | Verdict |
| ---- | ----------- | ------- |
| WorldGen.cs (29 sites) | `task/W4-g14-dust-worldgen-sites` pushed (0 commits yet) | CLAIMED — untouched here |
| Main.cs (24 sites) | `task/W4-g11/g12/g13` pushed; g12's census doc `64-dust-main-sites-claims.md` pins all 23/24 (the 24th, `CloneDust` :13410, rides g12's :13390 site) | CLAIMED — untouched here |
| d54 first-40 world lane | `task/W4-d54-dust-sites-world` exists, 0 commits, clean worktree, no claim doc | reserved below (the e2 precedent: d52's first-40 lane stayed untouched) |
| ParticleOrchestrator.cs | NO claim branch, NO claim doc; d60 defers orchestrator requests (`ParticleRequest` bytes); `prefix.rs` ports only RainbowRodHit/BestReforge RNG contracts (S1-P-i45), not the emitters | UNCLAIMED |
| Mount.cs | NO claim branch (minecart 213 sparks = S1-E-1, research/15 PARTIAL row) | UNCLAIMED beyond the 213 sparks |

## Census (independent re-derivation, `Dust.NewDust*` family only)

Counting convention = research/15's caller table: `NewDust` +
`NewDustPerfect` + `NewDustDirect` call sites; `CloneDust` rides its parent
site's ledger entry (it re-rolls a fresh `NewDust` inside, `Dust.cs:269`).

- `GameContent/Drawing/ParticleOrchestrator.cs` — **61** sites
  (16 NewDust + 37 NewDustPerfect + 8 NewDustDirect), 8 CloneDust siblings
  (:1214/:1225/:1638/:2144/:3333/:3387/:3436/:3486). research/15's "61"
  matches exactly.
- `Mount.cs` — **25** sites (14 + 7 + 4), 2 CloneDust (:6391/:6413).
  research/15's "25" matches exactly.
- `Main.cs` — 24 family hits = 23 NewDust* + 1 CloneDust (:13410); g12's
  census of 23 is the NewDust* count. All g11-g13's.
- `WorldGen.cs` — 29 (matches research/15). All g14's.

## Line-order split (the world lane)

The C#-line-ordered union of the two unclaimed world files is 86 sites.
The first 40 stay **d54's lane** (the first-batch branch, reserved here so
the second batch never overlaps it — the d52/e2 precedent):

- **d54 reserved: ParticleOrchestrator sites 1-40** — lines 423-2582
  (ArmletOfRuinRecharge through CopperSlimeEffect).
- **W4-e4 (this branch): the next 40 unclaimed** —
  ParticleOrchestrator sites 41-61 (21) + Mount sites 1-19 (19):

Whole C# methods stay whole (the golden rule ports the entire function);
TrueExcalibur carries 4 sites, so the orch split lands 11 + 10 and the
mount split 10 + 9 — 40 total.

| Batch | Sites | Lines | Arms |
| ----- | ----- | ----- | ---- |
| 1 (11) | orch 41-51 | 2653-3061 | ShimmerArrow pair, TerraBlade pair, Excalibur pair, ClassyCane, TrueExcalibur (4) |
| 2 (10) | orch 52-61 | 3195-3920 | TrueNightsEdge pair, PaladinsHammer, PrincessWeapon, StardustPunch, RainbowRodHit, LightningSmall, FormatC, FlameWaders, WoFGoatMountFlames |
| 3 (10) | mount 1-10 | 3113-4123 | drill block/wall sparks, unicorn dash ring + body, wolf ring + gated puff, witch ring + gated puff, basilisk charge pair + trail head |
| 4 (9) | mount 11-19 | 4626-5071 | golf-cart smoke, gem/steam/exhaust/confetti minecart, witch-broom pair, drill-mount trail pair |

Boundary sites owned whole: orch 2582 is d54's, 2653 is e4's; mount 5071
is e4's, 5099+ stays unclaimed for the next wave (f6 dust-world-3).
CloneDust siblings ride their parent: orch :3333/:3387/:3436/:3486 with
sites 54/55/56/57; mount :6391/:6413 stay with sites 23/24 (unclaimed).

## Contracts

- **Pool**: `tr_core::particles::DustPool::new_dust` / `new_dust_perfect` /
  `new_dust_direct` / `clone_dust` (d50's branch
  `task/W4-d50-dust-type-table`, `Dust.cs:75-256` verbatim). Post-spawn
  writes land on `slot_mut(idx)` — the sentinel 6000 takes them exactly
  like the C# (`Main.dust[6000]` exists).
- **RNG**: one slot per C# pass — the orchestrator spawns run SYNCHRONOUSLY
  inside `RequestParticleSpawn`/`BroadcastParticleSpawn`
  (`ParticleOrchestrator.cs:52-90` → `SpawnParticlesDirect` :162), so each
  emitter drains on the stream of the pass that invoked it (the caller
  passes `&mut UnifiedRandom`; never a new stream, S1-A-10). Mount frame/
  effect sites ride `UpdatePlayers`. Argument draws run BEFORE the pool
  internals (C# evaluates arguments first); `PrettySparkle`/`Flame`/
  `Fading`/`RandomizedFrame` particle-pool requests are RNG-free
  (`ParticlePool.RequestParticle`, Graphics/Renderers/ParticlePool.cs:32)
  but their ARGUMENT draws are consumed in order; the particle visuals
  themselves are the documented renderer skip (research/15: no particle
  renderer exists).
- **Gore siblings** (ClassyCane :2888, ElderSlime :2565) stay d55/g17's
  lane; their argument draws are consumed here in order (the e2 split).
- **RainbowRodHit/BestReforge overlap note**: `prefix.rs` (S1-P-i45)
  consumes those spawns' draws count-only on the reforge tail. This batch
  ports the FULL emitter bodies (site 57 + the d54-reserved BestReforge).
  **RESOLVED (the wiring wave, keep-one)**: the full
  `orchestrator_dust.rs::rainbow_rod_hit` owns every IN-WORLD site (the 79
  projectile hit `Projectile.cs:14234` + death `:71244`/`:71316`, the
  Kaleidoscope tag); the reforge tail KEEPS `prefix.rs`'s count-only drain
  as its ONE draw path. The two are disjoint by call site — no double-draw.
  The count-only form is exact under the empty-pool / on-screen assumption
  (`Dust.cs:97-152` cull + pressure bands draw nothing then; the reforge
  anchors on-screen at `MountedCenter`); a pool-aware fold would make the
  reforge stream occupancy-dependent and re-pin the frozen 602 witness, so
  it stays count-only (recorded in `prefix.rs` + `orchestrator_dust.rs` +
  ANALYSIS S1-P-i45). BestReforge stays d54's emitter lane; the reforge
  tail's count-only BestReforge drain (`prefix.rs`) is likewise its one
  path.
- WRITTEN NOT RUN (code-only mode): witnesses compile in the merge-verify
  wave, not executed here.

## Wiring wave (the call-site seams)

The emitters above are pure; the wiring gives them real call sites on the
caller-pass streams. The seams landed:

- **Orchestrator queue → drain** (`ProjectilePool::pending_orchestra` +
  `drain_orchestra`, `world_sim/projectiles.rs` pass tail): the C# spawns
  synchronously inside `RequestParticleSpawn`, but the pool cannot borrow
  the App's dust pool mid-walk, so the intents queue and drain on the SAME
  `UpdateProjectiles` stream (the S1-E-21c convention). The settings'
  ARGUMENT draws stay inline at the queue site (C# evaluates the settings
  initializer first); only the emitter bodies defer.
- **Per-type hit switch** (`damage_pve`, `Projectile.cs:12955-13000`): the
  `NextVector2FromRectangle(Hitbox)` settings draw (2 NextFloat, X first)
  fires for EVERY owned hit; 562 FormatC (crit-gated, +25% dmg), 973
  TrueNightsEdge, 984/985 TerraBlade, 982 Excalibur, 983 TrueExcalibur.
- **Post-strike chain** (`Projectile.cs:14227-14271`): 79 RainbowRodHit
  (NPC centre, movement = velocity), 1039/623 StardustPunch (the Y-jitter
  dir + movement-Y dir + magnitude float draws inline).
- **Kill arms**: 79 death (`:71314-71321`, 2 rotated RainbowRodHit), 1006
  ShimmerArrow (`:76050-76066`, one head NextFloat×2π). The 79's inline
  trail-dust chain (`:71267-71313`) + the 14 type-66 dusts are the
  projectile-dust lane's (the 932 count-only census precedent — NOT
  orchestrator sites 41-61). The `:71244` RainbowRodHit loop belongs to the
  **1047 RainbowBoulder** arm (`:71192-71256`), NOT the 79 — kind 1047 has
  no `proj_data` row and its AI is unported, so that call site is deferred
  with the boulder machine (the emitter is live via `spawn_orchestra`).
- **Tag effects**: SwordWhip MegaSlash (`tag_damage_for`, the rectangle
  draw + TrueExcalibur queue, tagged && proc-armed summon gate);
  Kaleidoscope OnTaggedHit (`tag_on_hit`, RainbowRodHit at projectile
  Centre, the C# `CanRunHitEffects` summon gate — distinct from the port's
  deliberately-ungated Constellation self-arm).
- **ClassyCane** (`editing.rs::resolve_melee_strike`, `Player.cs:44367-
  44375`): the coin velocity/position/`RollBadLuck(25)` draws inline on
  `UpdatePlayers`, then the emitter inline (editing has direct app access).
- **FlameWaders** (`apply_equip_functional` 4822/4874, `Player.cs:15174-
  15177`): the `DoBootsEffect` gate + tile-below probe queue the position;
  the player-tick drain rolls the emitter on `UpdatePlayers`.
- **Mount lane** (`MountState::update_frame_extras` + `update_mount`'s
  `UpdateEffects` cases → `MountDustRequest` queue → `mount_dust::
  drain_mount_request`): the unicorn frameExtra ramp + ring/body, the
  wolf/witch rising-edge latch rings + puffs, the basilisk charge ramp +
  puffs/trail (the FloorVisuals sandwich interleaved), the WoF goat's
  tile-gated flames, the witch-broom trinket, the gem/steam/exhaust/
  confetti minecarts and the drill-mount trail. New state: `dash_ring_latch`
  (the `_mountSpecificData` bool), `Player.basilisk_charge`, the dashing
  frame arm (case 5).

### Deferred (unported callers — documented, not invented)

- **AI_186 PrincessWeapon** (`Projectile.cs:42749-42784`): kind 950 has no
  `proj_data` row and aiStyle 186 is unarmed (the dispatch coverage pin
  `s1_d_4c_every_rowed_style_has_exactly_one_live_arm` scans the 24-space
  arms). The emitter (`princess_weapon`) is live via `spawn_orchestra`;
  the AI machine + data row are the projectile-AI lane's.
- **useStyle-5 PrincessWeapon** (`Player.cs:51165-51172`, item 5065): the
  useStyle-5 hold arm is unported; the emitter is live, the use-path call
  site waits on the hold-style machine.
- **NPC-side LightningSmall** (`NPC.cs:91150-91182` buff VFX +
  `PlayerDrawSet.cs:1217-1233`): the NPC buff-VFX pass and the draw-pass
  lightning are unported (no `markedByEelWhip`/`blueLightning`/
  `redLightning` NPC fields, no `canDisplayBuffs`). The `lightning_small`
  emitter is live via `spawn_orchestra` (Blue 37 / Red 77 both route to it,
  Red carries `GetLightningColor(1122).PackedValue` in info); the NPC
  callers wait on the buff-VFX pass.
- **Shimmerfly 677 death** (`NPC.cs:85097`): NpcKind 677 has no row; the
  `shimmer_arrow` emitter is live (the 1006 projectile death wires it), the
  NPC death caller waits on the kind row + the NPC death-dust pass.
- **UseDrill sparks** (`Mount.cs:3106-3173`): the drill mining arm
  (`UpdateDrill`, M2-MOUNT-4) is unported; `drill_sparks` is live, the
  mining call site waits on the drill machine. (The drill-MOUNT trail,
  `UpdateEffects` case 11, IS wired — it rides the mount tick, not mining.)
- **DEAD in C# (zero callers — documented, never wired)**:
  `ParticleOrchestraType.PaladinsHammer` (the enum value has no
  `RequestParticleSpawn` caller anywhere in the decompile — only the
  `SpawnParticlesDirect` dispatch arm) and `Mount.EmitGolfCartSmoke`
  (`Mount.cs:4623`, a private static with no caller). Their emitters
  (`paladins_hammer`, `golf_cart_smoke`) stay ported + reachable via
  `spawn_orchestra`/the module, but no call site is invented.
