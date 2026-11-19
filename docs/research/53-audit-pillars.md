# 53 — Celestial Pillars + DD2 lanes audit (census slice B-6)

**Mode:** READ-ONLY audit, code-only round — no cargo/rustc/rustfmt/tests/graphify
invoked; evidence is file reads + git only. Base: `origin/main` @ `c7fe175d`.
Census input: `docs/research/50-entity-ai-census.md` on branch
`origin/docs/entity-ai-census` (§B-6, rows 113/114/115/146/147, style rows 94/106).
Defect lens: the `f1701eb0` catalogue — missing gates, f32 op order, strict
bounds, invented constants, RNG draw order.

## 0. Id verification (the task's "517/518/519/520?" guess is wrong)

`decompiled/Terraria/ID/NPCID.cs`:

| Pillar | NPCID | line |
| --- | --- | --- |
| Vortex | **422** `LunarTowerVortex` | :11689 |
| Stardust | **493** `LunarTowerStardust` | :11659 |
| Nebula | **507** `LunarTowerNebula` | :11687 |
| Solar | **517** `LunarTowerSolar` | :11711 |

518 = `SolarSpearman` (:11713), 519 = `SolarGoop` (:11715), 520 =
`MartianWalker` (:11717) — mobs, not pillars. The pillar AI is **aiStyle 94**
(`NPC.cs:38968-39383`), NOT AI_084 (`AI_084_LunaticCultist` :64414 serves the
CultistBoss 439; the port already split that in S1-D-3b, `npc.rs:3941-3944`).

**Port id smell (pre-existing, not a merge loss):** the port's tower net ids are
**113/114/115/146** (`npc.rs:15312-15316` `to_u16`, `npc/data.rs:302-305` +
`:577-580` round-trip) — but vanilla 113/114/115 are **Wall of Flesh / WoF Eye /
The Hungry** (`NPCID.cs:10903-10907`) and 146 is `None3` (:10969). The save/net
round-trip is internally consistent, but any vanilla-id consumer
(`NPCAddHeight`, bestiary credit ids, `.wld` interop) reading these ids gets the
wrong row. The renderer dodges it by drawing the towers from dedicated
`tower_*` sheets keyed on the kind (`render/entities.rs:560-563`,
`NPC_493/507/517/422`), so pixels are right; the ids are still wrong. Same
family as ROADMAP S1-F-1 (protocol-id collisions). **Fix spec:** remap the four
`to_u16`/`from_u16` rows to 422/493/507/517 (and the `npc/data.rs` twins) with a
save-migration for the old 113/114/115/146 values; verify no other kind claims
422/493/507/517 first.

## 1. Verdict table

| # | Target | Verdict | Evidence |
| --- | --- | --- | --- |
| 1a | Pillar live machine (aiStyle 94 body) | **WRONG** | `npc.rs:12553-12606` vs `NPC.cs:38968-39383` — see §2 |
| 1b | Shield lifecycle state (raise/reset/max/save) | FAITHFUL (state only) | `world_sim/npcs.rs:1570-1608`, `lunar.rs:39-90`, `wld/sections/flags.rs:515-531`, `save.rs:503-507` |
| 1c | Shield **wiring** into the live sim | **MISSING** | `tower_shield_absorb`/`update_tower_shields` have no live callers (tests only, `npcs.rs:9557-9606`); towers never read `tower_shield_*` |
| 1d | Pillar death → apocalypse chain | FAITHFUL | `npcs.rs:3360-3406` vs `NPC.cs:79321-79345` + `WorldGen.cs:86626-86760`; `lunar.rs:124-172` |
| 1e | `update_lunar_apocalypse` slot table | **WRONG** | `lunar.rs:135-136` duplicate `NebulaTower` arm — Vortex never counted |
| 1f | Pillar → celestial mob spawn wiring | **MISSING** (all four arms) | no `SolarGoop` kind, no tower-side `SpawnStardustMark`, vortex arms fire at the wrong target with no cooldown; §2.3 |
| 1g | Tower dissolve (`checkDead` intercept + ai[2] arm) | **MISSING** | `NPC.cs:78080-78086` + `:38970-39067`; port has neither |
| 2a | Tower projectile 629 arm (aiStyle 122) | DEVIATION-DOCUMENTED (arm) / **WRONG** (spend) | `projectile.rs:3313-3342` + `:13502-13521` vs `Projectile.cs:34896-34965` + `:72944-72979`; §3 |
| 2b | 629 spawn sites (mob deaths) | **MISSING** | `NPC.cs:79279-79320`; no live spawner of `TOWER_DAMAGE_BOLT` anywhere |
| 2c | Ritual markers 578/579 (aiStyle 108) | DEVIATION-DOCUMENTED + 2 defects | `projectile.rs:3134-3203` vs `Projectile.cs:33679-33988`; §3.2 |
| 3a | DD2 lane gate machine (aiStyle 106) | DEVIATION-DOCUMENTED (1 WRONG sub-item) | `npc.rs:10731-10776` vs `NPC.cs:40970-41047`; §4.1 |
| 3b | Lane spawn positions / portal creation | **MISSING** | crystal never spawns the two 549 portals (`NPC.cs:40744-40755`); `editing.rs:7960-7961` admits "no lane machinery" |
| 3c | Lane spawn rolls + rates + crystal guard | FAITHFUL (tables) / DEVIATION-DOCUMENTED (RNG + one spawn-drop) | `dd2.rs:27-36/205-234/390-473`, `npcs.rs:1952-2035/3850-3863`; §4.2-4.3 |

## 2. Target 1 — the pillar machine (WRONG + MISSING)

### 2.1 What the port runs

All four towers dispatch to `update_ancient_cultist` (`npc.rs:3936-3940`) —
the census' "towers ride a cultist-named fn" smell is **confirmed and worse
than a naming issue**: the fn body (`npc.rs:12553-12606`) is a ~50-line
stand-in, not the C# arm. Line-by-line against `NPC.cs:38968-39383`:

| C# (NPC.cs) | Port | Defect class |
| --- | --- | --- |
| `:38970-39067` ai[2]==1 dissolve: velocity→UnitY, ±0.02 clamp to 0.25, dontTakeDamage, opacity fade after 120, per-type dust (127/229/242/135), death at ai[1]≥180 | `npc.rs:12555-12558` counts **ai[2] as a timer** (`ai[2] += 1; >= 180 → hp = 0`) — ai[2] is the C#'s **dissolve flag**, set only by `checkDead` (`:78080-78086`). Every tower self-destructs 180 ticks after spawn, unprovoked | invented constant + wrong state semantics |
| `:39069-39100` ai[3] shield-break flash (NPCDeath58 on flip, NPCDeath3 at ai[3]==1, 120-wrap) | absent | missing gate |
| `:39101-39115` `dontTakeDamage = ShieldStrengthTower<Kind> != 0` per type | absent — the doc comment `npc.rs:10356-10357` still says "no pillar shields yet" although the state landed (`world.rs:347-353`) | missing gate (the shield never protects) |
| `:39116-39130` TargetClosest + `localAI[0]` regen: +200/60 ticks **only while distance > 2000**, counter resets otherwise | `npc.rs:12569-12579` matches the shape (hypot > 2000, 60-tick, +200 clamp) — faithful fragment | — |
| `:39131` sine bob `sin(2π·ai[0]/300)·0.5`, ai[0] 300-wrap `:39190-39195` | `npc.rs:12563-12568` matches | — |
| `:39132-39153` ground hover: `WorldUtils.Find` Down(10) solid → `Y -= 1.5·(1-|ΔY|/10)`; else Down(20) miss → `Y += 1.5·lerp(20,30)` | absent — towers float at spawn height forever | missing gate |
| `:39154-39189` world-edge clamps (60-tile margin + 80 px, `maxTilesX*16` on **both** axes — the C# quirk), no-surface/remix/getGoodWorld arms, `worldSurface*16 - height - 100` floor | absent | missing gates (the Y clamp uses the X extent in C# — port must copy the quirk, not "fix" it) |
| `:39196-39224` type 493: dust + ai[1] cooldown decrement + `SpawnStardustMark_StardustTower()` when player < 1080 px and `player.Y - tower.Y < 400` | absent (see §2.3) | missing |
| `:39226-39247` type 507: dust only (no attack arm — nebula mobs come from zone spawns) | absent (dust unmodelled is fine; nothing else to port) | — |
| `:39249-39345` type 422: ai[1] cooldown; **3240-px no-LOS arm** `ai[1] = 60 + Next(120)`, count gate `CountNPCS(428)+CountNPCS(427)+CountNPCS(426) < 14` → 579 portal at the player's **top tile scanned up ≤10 rows** (`point.Y*16+24`), else 578 lightning (`+17`, ai1=1); **1080-px arm** `ai[1] = 420 + Next(360)`, count gate `CountNPCS(427)+3·CountNPCS(426)+CountNPCS(428) < 20`, the 100-try `Next(playerTile±20)` window pick with the 3/8-tile exclusion boxes, lava/solid/LOS gates → 579 | `npc.rs:12580-12604`: fires 579 **at the player centre every tick** (no ai[1] cooldown at all), no count gates, no tile scan, no 578 branch, no window pick; the "420+360 rearm" comment cites a constant the code never writes (`caster_counter = 420` is the regen counter, not ai[1]) | missing gates + RNG draw order (the C#'s `Next(120)`/`Next(360)`/window draws never happen) + invented targeting |
| `:39347-39382` type 517: dust + ai[1] cooldown; player < 1080 px and `ΔY < 700` → **NewNPC 519 SolarGoop** at the tower-top random point, velocity `-UnitY.RotatedByRandom(π/4)·(7+NextFloat·5)`, `ai[1] = 60` | absent — **no `SolarGoop` NpcKind exists** (only the lang row `lang_tables.rs:596`) | missing kind + missing arm |

The fn also takes `rng` and discards it (`npc.rs:12605 let _ = rng;`) — the
C# arm is RNG-heavy (dust gates + the rearm/window draws); the port's tower
ticks draw **zero** RNG, so any downstream stream alignment differs from
vanilla the moment a tower is alive.

### 2.2 Shield state vs shield wiring

Landed and faithful **as state**: `raise_tower_shield` / `tower_shield_max`
(post-Moon-Lord halving, `NPC.cs:6795-6806`) / the all-dead reset
(`NPC.cs:95247-95251`) at `npcs.rs:1570-1608`; the trigger raise
(`WorldGen.cs:86688`) at `lunar.rs:86-89`; the `.wld` restore
(`flags.rs:515-531`) and save round-trip (`save.rs:503-507`).

**But nothing live consumes it:** `tower_shield_absorb` (`npcs.rs:1615-1652`)
and `update_tower_shields` are called only from tests (`npcs.rs:9557-9606`,
`:11490`). The tower machine never sets `dont_take_damage`, the strike path
never drains a shield, and the 629 bolt's spend (§3) is the only live shield
writer — for bolts that are never spawned (§2.3/3.2). Net effect: **pillars
are killable by direct damage from tick 1 and die by themselves at tick 180.**

`tower_shield_absorb` itself is also a wrong model even if wired: it drains
`dmg` per hit and flips `downed_tower_*` at zero. The C# never drains shields
via NPC damage — the shield falls only through **629 bolt contacts, −1 per
bolt** (`Projectile.cs:72953`), and `downedTower*` is set by the tower's
**death** (`NPC.cs:79321-79345`), not by shield exhaustion. Fix spec: retire
`tower_shield_absorb`'s downed-flag writes; the death arms at
`npcs.rs:3360-3391` already own them.

### 2.3 Pillar → mob spawn wiring (the census' "celestial mobs landed" check)

The mob **kinds and machines** are landed (census rows 116-145: stardust
cells/jellyfish/spiders/soldier/worm, nebula beast/floater/suckler/predictor,
vortex rifleman/soldier/hunter/alien/headcrab, solar drakomire/rider/sroller/
corite/selenian/spearman/crawltipede — dispatch `npc.rs:3956-4009`). The
**pillar-driven spawn wiring is not**:

1. **Solar (517 → 519 SolarGoop):** MISSING entirely — no kind, no arm
   (`NPC.cs:39374-39382`). Port-spec below.
2. **Stardust (493 → mark tree):** MISSING — `SpawnStardustMark_StardustTower`
   (`NPC.cs:43821-43907`) has no port twin. The port's `wants_stardust_mark`
   (`npc.rs:1423-1425`, `:11722`) is the **worm's** mark
   (`SpawnStardustMark_StardustWorm`, `:43909+`), a different fn.
3. **Vortex (422 → 578/579 markers):** the markers' own machine is landed
   (`ai_vortex_ritual`, §3.2) but the tower fires them wrong (§2.1) — no
   cooldown, no count gates, wrong position, no 578 branch. The 579's payload
   (two VortexAliens 427) is the only pillar→mob path that can fire today,
   and it fires every tick at the player's centre.
4. **Nebula (507):** no direct spawn arm in C# either (dust only,
   `:39226-39247`) — nebula mobs arrive via the celestial **zone spawn pool**
   (`Main.cs`'s SpawnNPC / `NPCID.Sets`... the port's `biome.rs` has no
   celestial/tower zone at all — grep `celestial|tower` in `biome.rs`: zero
   hits). That is a separate spawn-system slice, noted here as MISSING
   context, not a pillar-machine defect.

### 2.4 The `update_lunar_apocalypse` slot bug (WRONG, one-line fix)

`lunar.rs:133-143`:

```rust
NpcKind::SolarTower => 1,
NpcKind::NebulaTower => 2,
NpcKind::NebulaTower => 3,   // <-- duplicate arm; C# slot 3 is Vortex
NpcKind::StardustTower => 4,
```

The second `NebulaTower` arm is unreachable (rustc warns
`unreachable_patterns`); `alive[2]` (vortex) is **never set**, so
`:148 tower_active_vortex = alive[2]` clears the vortex flag on the first
update after any tower death, and the all-four-down doom check
(`:151`) can fire while the Vortex pillar still stands. Fix: `NpcKind::VortexTower => 2`
(or 3 — keep the table consistent with `:147-150`; the C#
`WorldGen.cs:86693-86748` counts per-type, slot order is the port's own).
The death arms (`npcs.rs:3360-3391`) set the flags correctly first, but this
fn re-derives them from the live roster every call and overwrites.

### 2.5 Port-spec for the next wave (MISSING pillar machines)

New module `npc_ai_pillar.rs` (house style: pure fn over `Npc` + a context
seam, cf. `npc_ai_style29.rs`), dispatch from `npc.rs:3936-3940`; delete the
tower arms of `update_ancient_cultist` (keep the fn for nothing — the real
cultist already rides `update_lunatic_cultist`; rename/retire after the
switch). C# ranges to port, in arm order:

1. **Dissolve intercept** — `NPC.checkDead` `NPC.cs:78080-78086`: on hp≤0
   with ai[2]≠1 → `ai[2]=1, ai[1]=0`, keep alive (the port's death pass
   `npcs.rs:3352+` needs the intercept before the retain, same shape as the
   crystal's `:78087-78095` intercept already landed at `npcs.rs:2094-2106`).
2. **Dissolve arm** — `NPC.cs:38970-39067`: velocity→UnitY·length, the ±0.02
   clamp toward 0.25, dontTakeDamage, ai[1]++, opacity `1-(ai[1]-120)/60`
   past 120, death at 180 (`HitEffect(0,1337)` + checkDead). Dust/sound arms
   documented-deferred, but **the RNG gates must still draw** if the port
   models dust later; for now record the zero-draw deviation explicitly.
3. **Shield gates** — `:39069-39115`: ai[3] flash cycle + per-type
   `dont_take_damage = world.tower_shield_<kind> != 0`.
4. **Hover/regen/clamps** — `:39116-39195`: TargetClosest
   (`npc_target::target_closest`), the >2000-px regen, the sine bob, the
   Down(10)/Down(20)/Down(30) ground hover, the edge clamps **including the
   C#'s `num1302 = maxTilesX*16` Y-axis quirk**, the worldSurface floor.
5. **Per-type attack arms** — 493 `:39196-39224` + the mark tree
   `:43821-43907` (BFS over `list2`, the 540 mark projectiles with
   `ai0=-depth*10, ai1=0.5+NextFloat·0.5`, the carried kind picked by
   `SelectRandom` from the count-gated list [405 if cells<2, 402 if <2,
   407 if <1], `ai[1]=30·Next(5,16)`, the empty-list `ai[1]=30` fallback);
   422 `:39249-39345` (both arms verbatim: cooldowns `60+Next(120)` /
   `420+Next(360)`, the count gates, the top-tile up-scan, the 100-try window
   pick with its exclusion boxes and lava/solid/LOS gates); 517
   `:39347-39382` (**new kind SolarGoop 519** + its aiStyle-3 fighter row —
   `NPC.cs` SetDefaults case 519 — and the goop's own machine, a follow-up
   census row); 507 dust-only.
6. **Shield spend + bolts** — see §3 fix spec.
7. **Death chain** — already landed (`npcs.rs:3360-3391`); keep. Fix the
   `lunar.rs:135-136` slot arm in the same wave.

RNG order note: the C# arm interleaves dust-gate draws with the attack-arm
draws per tick; a zero-dust port must decide and document whether it draws
the gates anyway (the `f1701eb0` lesson: draw order is behaviour). Recommend
drawing the gates in C# order even with dust unmodelled, as
`npc_ai_bat.rs` does for its deferred arms.

## 3. Target 2 — tower projectiles

### 3.1 The 629 TowerDamageBolt (aiStyle 122)

**Arm** (`projectile.rs:3313-3342` vs `Projectile.cs:34896-34965`): the 629
half is close — host-dead kill (`:34900-34910`), the 5-px normalize homing
(`:34929`), the hitbox contact kill (`:34917-34925`). Deviations: the port
uses a centre-box test (`projectile.rs:3337`) instead of
`Hitbox.Intersects` (the bolt's own 8×8 box vs the tower's 130×270 — the
port's test ignores the bolt's extent, a strict-bounds slip of a few px);
the `ai[0]==-1` sentinel is folded into `npcs.get()` (fine); the red
QuickDust is unmodelled (documented). The 631 half of the style (the
DD2 ballista-ish variant) is absent — separate slice.

**Spend** (`projectile.rs:13502-13521` vs `Projectile.cs:72944-72979`):
**WRONG on three counts** —
1. drains `slot.damage` per contact; C# drains **−1** per bolt
   (`:72953 MathHelper.Clamp(ShieldStrengthTowerSolar - 1, 0, Max)`),
   regardless of damage (the 629 spawns with damage 0, `NPC.cs:79290`);
2. selects the shield by `ai[0] as usize` **0/1/2/3**; C# switches on
   `Main.npc[(int)ai[0]].type` (517/422/507/493) — `ai[0]` is the **host NPC
   index**, not a tower ordinal. With the port's compacted vec the index is
   whatever slot the tower occupies, so the spend hits an arbitrary shield;
3. never sets the host's `ai[3] = 1` shield-flash trigger (`:72951` etc.).

**Spawn sites: MISSING.** `NPC.cs:79279-79320` (inside `checkDead`'s
pre-death block): solar mobs 412-419+518 → 629 at the mob centre with
`ai0 = FindFirstNPC(517)` while `ShieldStrengthTowerSolar > 0`; vortex
425/426/427/429 → host 422; nebula 420/421/423/424 → host 507; stardust
402/405/407/409/411 → host 493. No live code spawns `TOWER_DAMAGE_BOLT`
(grep: only the data rows `projectile/data.rs:1480/1888` and tests). This is
the **kill-count machine** the task asked about: each celestial mob killed
while its pillar's shield stands sends one bolt that chips the shield by 1;
~100 kills (the `LunarShieldPowerNormal` default, `NPC.cs:95251`) drop the
shield. The port has the counter state but not the loop.

**Fix spec:** in the death pass (`npcs.rs:3352+`, before the tower arms),
per dead celestial mob of the four id sets, if the matching
`world.tower_shield_* > 0` push a 629 at the mob centre with the host's
**current vec index** (the port's `wants_hostile_shot_ai` seam already
carries ai0/ai1); on contact spend −1 (clamp 0..max) and set the host's
`ai[3]=1`; the shield-zero → `dontTakeDamage=false` transition then rides
the new pillar machine's per-tick gate (§2.5 step 3), with the NPCDeath58
cue on the flip.

### 3.2 The 578/579 ritual markers (aiStyle 108) — the "towers under cultist fn" citation

The census flagged the towers riding `update_ancient_cultist`; the
projectile side is **correctly separated**: `ai_vortex_ritual`
(`projectile.rs:3134-3203`) serves 578/579 under its own name, dispatch
`projectile.rs:13528-13535`, data rows `projectile/data.rs:1483/1891`. The
timeline (≤50 wind-up / ≤90 charge / ≤120 full / dissolve, the rotation
decrements π/20, π/60, π/30, the 578's aim latch, the 90th-tick 580 bolt at
`ai1=aim`) matches `Projectile.cs:33679-33988`'s shape. Two defects in the
579 payload (`projectile.rs:3192-3200` vs `Projectile.cs:33844-33856`):

1. **RNG draw order / invented constant:** C# velocity is
   `-UnitY.RotatedByRandom(2π) * Main.rand.Next(4, 9) - UnitY*2` — an
   **integer** magnitude 4..8. The port draws `4.0 + next_float()*5.0`
   (continuous 4..9). Different distribution and a different draw shape.
2. **Spawn drop:** the loop writes `self.wants_mark_spawn = Some(...)`
   **twice into one Option** — the first VortexAlien is overwritten; C#
   spawns **two** (`for num815 < 2`). The seam needs a Vec (or two fields).

Also the 580 bolt's `ai1: Main.rand.Next(80)` (`Projectile.cs:33841`) is not
drawn by the port's `wants_ritual_bolt` payload (`projectile.rs:3190`) —
check the resolver; if it defaults ai1=0 the bolt's variation is lost
(flagged, not fully traced — the resolver lives in the pool pass).

## 4. Target 3 — DD2 lanes

### 4.1 The lane gate machine (aiStyle 106)

`npc.rs:10731-10776` vs `NPC.cs:40970-41047`. The citation smell is
**resolved**: the fn's doc comment (`npc.rs:10719-10730`) cites the correct
`:40970-41047` range; the census' "cites AI_077" flag is stale (AI_077's
range belongs to the neighbouring `update_moon_lord_core`,
`npc.rs:10778-10786`). Faithful fragments: the hold-parked counter
(`:40988-40991` → `enemy_spawning_on_hold`, `npcs.rs:1968-1970`), the
rate/3×-reset/modulo cadence **including the reset-frame `0 % rate == 0`
attempt** (`:40994-40999` → `npc.rs:10740-10748`), the 180-clamped idle
timer, the crystal guard (`AnyNPCs(548)` → `world.dd2_crystal_alive`,
refreshed each tick at `npcs.rs:2135-2138`), the fade and the 550 death.

Defects:

1. **WRONG — fade window:** `npc.rs:10769` uses
   `((ai[0]-500)/100).clamp(0,1)`; C# `:41028` is
   `Lerp(1, 0.05, GetLerpValue(500, 600, ai[0], clamped))` — a **500→600**
   ramp. The port finishes shrinking at 550 (the death tick) instead of
   still mid-fade; the scale at death is 0.05 in the port vs 0.525 in C#.
   Invented constant (100 vs the 600 endpoint).
2. **Missing gate (minor):** C# `:41002-41005` — after a spawn, if
   `EnemySpawningIsOnHold` became true, `ai[0]++` (so the next attempt
   skips a frame). The port's `wants_dd2_spawn` deferral
   (`npcs.rs:3850-3863`) resolves the spawn in a later pass and never
   feeds the hold-back increment. One-frame cadence slip under hold
   transitions.
3. **Documented skips:** the portal-open/idle-loop tracked sounds
   (`:40981-40986`, `:41029-41037`), the `Lighting.AddLight(0.5,0.1,0.3)`
   aura (`:40975`), netUpdate. The light is the "crystal aura" the task
   asked about on the **portal** side — unmodelled, no light seed. The
   crystal's own aura (`:40732 AddLight(1.3,0.5,1.5)`) is likewise absent
   from `update_eternia_crystal` (`npc.rs:10287-10344`).

### 4.2 Lane spawn positions — MISSING (the big one)

C# creates the two lane portals from the **crystal's** aiStyle-105 arm
(`NPC.cs:40744-40755`): at `ai[0]==0` (180 ticks after spawn) it runs
`StrayMethods.CheckArenaScore(base.Bottom, out xLeftEnd, out xRightEnd)`
(`StrayMethods.cs:83-105`: `Collision.ExpandVertically` to the floor, then
`SendWalker` both directions — the 3-up solid skip, the height-10
`ExpandVertically` walkability test, the 6-down floor probe, maxDistance =
`MaxWorldViewSize.X/16`), nudges the ends ±2 tiles, spawns **549** at each
end with `Bottom = end.ToWorldCoordinates(8,16)`, the right one with
`ai[2]=1` (the `leftGate` flag the lane machine reads), then
`DD2Event.FindArenaHitbox()` (`DD2Event.cs:753`).

The port: `use_etherian_crystal` (`editing.rs:7963-8022`) spawns **only the
crystal** and its own comment admits the fold — "WouldFailSpawningHere folds
to the stand-tile check (the port has no lane machinery — the DD2Event
framework's slice)" (`:7960-7961`). `update_eternia_crystal`
(`npc.rs:10287-10344`) has no `ai[0]==0` portal-spawn arm (its live body is
the 180-tick spawn shield + a 600-tick counter that stops the event — itself
a stand-in for the C#'s win-scene-driven stop). **No live code creates a
`DD2LanePortal`** — every construction site is a test
(`npcs.rs:12047/12062` etc.). So the whole landed lane machinery
(`update_dd2_lane`, `spawn_dd2_from_gate`, the three tier roll tables) is
**unreachable in a real play session**: the event starts, the crystal
stands, and nothing ever spawns from lanes.

Also missing: `WouldFailSpawningHere` (`DD2Event.cs:603-613`, the 60-tile
each-side arena check) and the crystal's win-scene arm (`ai[1]==2`,
`NPC.cs:40758-40860` — the portal fade marks at `ai[0]==3`, the 713
projectile, the cos-hover, the 600-tick stop). The loss scene IS landed
(`npcs.rs:2093-2118`).

**Port-spec (next wave):** in `update_eternia_crystal`'s live branch port
`:40734-40756` (the ai[0] countdown → 180 rearm → the one-shot
`localAI[0]` portal spawn); port `CheckArenaScore`/`SendWalker`
(`StrayMethods.cs:83-156`) as a pure fn over `World` (the port has
`Collision.ExpandVertically`-shaped helpers in `npc_ai` — check before
duplicating); spawn the two portals through a `wants_lane_portals` seam
(the machine holds `&World`, same pattern as `wants_dd2_spawn`), resolved
in `npcs.rs`'s spawn pass with `Bottom` anchoring and `ai[2]=1` on the
right; `FindArenaHitbox` (`DD2Event.cs:753-...`) feeds the crystal's
hit-detection box — port with the arena rect or document the fold.

### 4.3 Lane rolls, rates, crystal guard — tables faithful, two flags

- **Rates:** `dd2.rs:27-36` (90/60/55/50/40, default 60) matches
  `DD2Event.cs:953-975`; t2 `:205-214` (90/70/60/55/50/45/42) matches
  `:1132-1156`; t3 `:220-234` (85/75/60/60/55/60/90) matches
  `:1437-1466`. `lane_spawn_rate` (`npcs.rs:1952-1964`) picks by
  `find_tier` — faithful.
- **Rolls:** `lane_spawn_roll_d1` (`dd2.rs:390-473`) matches
  `DD2Event.cs:1011-1105`'s caps (50 / 6→8→12 / 6→8), the wave ladder and
  the short-circuit draw discipline (only the taken branch draws). Two
  flags:
  1. **DEVIATION-DOCUMENTED (RNG):** the rolls consume `next_float()` and
     test `< 1/N` where C# draws `Next(N)==0` (`dd2.rs:386-389` admits it).
     Different stream consumption shape — acceptable only as a recorded
     deviation; it is recorded.
  2. **WRONG (spawn drop, wave 5):** `dd2.rs:463-469` returns
     `chain.or(boss)` — one body. C# `:1078-1093` executes the Dark Mage
     `NewNPC` **and then** the chain's `NewNPC`; `num4` is only the
     net-sync handle, both NPCs spawn. The port drops the mage whenever
     the chain fires. Fix: `LaneSpawn { first: boss, second: chain }`
     (the pair seam already exists, `dd2.rs:373`). The comment at
     `:463-465` misreads the C# — correct it in the same fix.
- **Spawn position:** `npcs.rs:3854-3856` uses `(n.x, n.y + nh)` = the
  gate's bottom-left; C# `SpawnMonsterFromGate(base.Bottom, ...)` →
  `Difficulty_1` casts `(int)gateBottom.X/Y` and `NewNPC` centres the body
  on that point (`NPC.NewNPC` spawns at x−width/2). The port's
  `Npc::new(kind, sx, sy, ...)` anchoring needs the same centre fold —
  verify `Npc::new`'s convention; if it treats (sx,sy) as top-left the
  bodies spawn half a gate-width left of vanilla. Flagged for the wiring
  wave (not fully traced here).
- **Crystal guard:** faithful (`npc.rs:10755-10765` vs `:41013-41023`),
  fed by the live scan `npcs.rs:2135-2138`.

## 5. Summary for the Lead

- **Do not trust the pillar fight today:** towers self-kill at tick 180,
  shields never gate damage, no mob kills chip shields (no 629 spawns),
  solar/stardust pillars spawn nothing, vortex spam-fires portals at the
  player every tick. The apocalypse chain and the shield *state* are the
  only faithful pieces.
- **One-line WRONG fix available now:** `lunar.rs:136`
  `NebulaTower => 3` → `VortexTower => 3` (unreachable-pattern warning +
  premature doom).
- **DD2 lanes are dead code in live play:** the crystal never spawns the
  portals; the arena walker (`StrayMethods.cs:83-156`) is the port-spec's
  core. The lane machine/rolls/rates themselves are in good shape (one
  fade-window constant, one wave-5 spawn drop, the documented float-RNG
  deviation).
- **Id remap** (towers 113/114/115/146 → 422/493/507/517) joins the S1-F-1
  collision family; needs a save migration.
- Next-wave slices suggested: **B-6a** pillar machine re-port (§2.5, L),
  **B-6b** 629 spend/spawn loop + shield wiring (§3.1, M), **B-6c** DD2
  portal creation + arena walker (§4.2, M), **B-6d** the small WRONGs
  (lunar slot, fade window, wave-5 pair, ritual double-spawn, id remap
  groundwork) (S each).

*Audit performed read-only on `c7fe175d`; no compiler invoked. All line
numbers are against that base and the junctioned decompilation.*
