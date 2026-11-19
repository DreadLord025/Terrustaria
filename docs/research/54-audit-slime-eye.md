# 54 — Census slice A-1 audit: the most-played early-game AIs (slime / demon eye / fighter / King Slime)

Distrust-verbatim audit of the four highest-exposure live AI arms, per census §A-1
(`origin/docs/entity-ai-census:docs/research/50-entity-ai-census.md`). Method: whole C#
arm re-read against the live Rust arm line-by-line, f1701eb0 defect catalogue applied
(missing gates, f32 op order, int-vs-float division, strict bounds, draw-before-early-
return, invented constants, RNG draw order/count). **Code-only round — no compiler
invoked, no tests run; reads + git only.** Branch `docs/audit-slime-eye`, base `c7fe175d`.

C# = `decompiled/Terraria/NPC.cs` unless noted. Rust = `crates/tr-app/src/` unless noted.

## Verdict table

| # | Target | Live arm | C# arm | Verdict |
|---|--------|----------|--------|---------|
| 1 | aiStyle-1 slime family (10 kinds, `npc.rs:3744-3753`) | `npc.rs::update_slime` `:6652-6731` | `AI_001_Slimes` `:60341-61723` | **WRONG** — stand-in machine, not a port |
| 2 | aiStyle-2 eye family (7 kinds, `npc.rs:3754-3771` → `:3760`) | `npc.rs::update_eye` `:6733-6772` | `AI_002_FloatingEye` `:52274-52731` | **WRONG** — stand-in; lens drops FAITHFUL (separate) |
| 3 | aiStyle-3 fighter base (11 zombie kinds, `npc.rs:3761-3771`) | `npc.rs::update_zombie` `:6775-6876` | `AI_003_Fighters` `:55856-60298` | **WRONG (rest of arm)** — target core + speed ladder FAITHFUL; door bash / dawn / jump tiers / burn wrong or missing |
| 4 | King Slime 50 (`npc.rs:4064`) | `npc.rs::update_king_slime` `:7290-7454` + `king_slime_find_teleport_spot` `:1772-1827` + minion drain `world_sim/npcs.rs:4280-4304` | `AI_015_KingSlime` `:43121-43414` + `:43056-43119` | **DEVIATION-DOCUMENTED core, WRONG defects** — structure faithful, 4 concrete defects incl. a never-despawn gate bug |

Exposure: targets 1-3 run for essentially every surface session (Green/Blue Slime,
Demon Eye, Zombie are the three most-spawned mobs); target 4 gates the first boss.

---

## 1. Slime family — WRONG

The live arm is a **sin-timer hop stand-in**, exactly the census §A-1 prediction. None
of the C# `ai[]` machine exists.

### 1a. Invented jump cadence (the headline defect)
- Rust `npc.rs:6660-6668`: `hop_timer -= 1/60`; jump when `on_ground && hop_timer <= 0
  && |pcx-cx| < 60*TILE_PX`; writes `vx = facing*1.4`, `vy = -4.2`,
  `hop_timer = 1.0 + |sin(anim*7)|*0.5`.
- C# `:61511-61703`: the cadence is the **`ai[0]` tick counter**, no RNG and no sine:
  init `ai[0] = -100` (`:61512-61517`, with `TargetClosest`), grounded `ai[0]++`
  (`:61554`, doubled by `flag3` shooter kinds `:61550-61553`), per-type bonuses
  (`:61555-61602`), tiers `num51` at `ai[0] >= 0` / `[-1000,-500]` / `[-2000,-1500]`
  (`:61603-61624`), hop `vy = -6` / `-8`, `vx += 2*direction` / `3*direction`, resets
  `ai[0] = -120 (+num50 or 2*num50)` / `-200` (`:61632-61664`).
- **Every constant in the Rust arm is invented**: 1.4 / -4.2 / 60-tile aggro /
  1.0+0.5·sin period. The C# has **no distance gate at all** — slimes hop toward
  `base.direction` from `TargetClosest` regardless of range. Note for the fix wave:
  the C# jump cadence draws **zero** `Main.rand` values (the census's "jump rolls"
  worry is unfounded for the cadence itself; the arm's draws are the item-content
  init `:60356-60575` and dust/gore/audio, all spawn-time or cosmetic).
- Fix spec: port `ai[0]`/`ai[2]`/`ai[3]` verbatim (`:61459-61723`), the stuck-flip
  `ai[3] == position.X → direction *= -1; ai[2] = 200` on **both** the wet branch
  (`:61469-61473`) and the grounded branch (`:61524-61528`), the air steer
  (`:61704-61721`: `+0.2*direction` under 3 cap else `×0.93`, the `collideX &&
  |vx|==0.2` 1.4-px nudge), and the landing brake `vx *= 0.8` with the f64 ±0.1
  deadband (`:61544-61548`) — the Rust landing `vx *= 0.6` (`npc.rs:6694`) and wall
  bounce `vx = -vx*0.4` (`:6682`) are invented (the C# generic pass zeroes `vx` on
  `collideX`; `AI_001` never negates it).

### 1b. Gravity / wet / honey merges
- Rust `npc.rs:6671-6676`: `is_wet` (any `liquid > 0`, `npc.rs:16-29`) → `vy+0.2` cap 7,
  else `GRAVITY = 0.4` cap `MAX_FALL = 8.0` (`npc.rs:12-13`).
- C# `UpdateNPC_UpdateGravity` `:90878-90955`: default **0.3** × the depth factor
  `num2` (`:90926-90936`, clamp [0.25,1]); wet water 0.2/7 (matches), **honey 0.1/4**
  (`:90944-90948`) and **shimmer 0.15/5.5** (`:90939-90943`) — both missing; the port
  treats honey/shimmer as water. Dry 0.4/8 is an invented constant pair (wrong
  gravity + wrong cap, no depth scaling).
- The wet buoyancy tail (`npc.rs:6704-6723`) is roughly the C# `:61463-61505` shape
  (collideY → -2, ×0.9 over 2, -0.5/tick, cap -4) but the stuck-flip writes
  `hop_timer = 200/60` seconds instead of `ai[2] = 200` ticks gating the
  `ai[2] > 1 → ai[2]--` countdown (`:61459-61462`) and the `ai[2]==1 & flag3 →
  TargetClosest` re-aim (`:61506-61509`) — the countdown semantics are lost.

### 1c. Missing per-type arms and gates (whole-arm coverage)
- **Item-content init missing** (`:60356-60575`): `SlimeCanContainItems` + `value > 0`
  → the 1/20 (1/40 bonus) rolls that put potions/ores/balloons inside slimes
  (`AI_001_Slimes_GenerateItemInsideBody` `:61734-61848`), the `slimeRain` +2 rolls
  (`:60376-60379`), the Black-Slime netID -6 +4 rolls (`:60372-60375`). This is a
  large **RNG draw-count deficit at spawn** and a loot-feature hole (gel itself drops
  fine via `loot_db_gen.rs:469` `register_to_npc(1, gel(1,1,2))` — FAITHFUL, matches
  `ItemDropDatabase.cs:1074-1084`).
- **LavaSlime (59)**: the +2/tick cadence (`:61555-61558`), the -8-2 jump / +3.5 vx
  (`:61634-61643`), the wet -0.8 lift / cap -10 (`:61478-61493`) — all missing; the
  port routes it through the generic stand-in (`npc.rs:3746`).
- **CorruptSlime (81)**: +4 cadence (`:61592-61602`) missing.
- **SpikedIceSlime (184)**: the whole spike-shooter block (`:61151-61210` —
  `flag3`, `localAI[0]` cooldown, the <200 px `CanHit` lob of projectile 174 with
  `Next(0,200)` Y-jitter, expert 5-spike fan) missing.
- **IceSlime (147)** dust `Next(10)` (`:61101-61108`) — cosmetic skip, but the draw
  must be counted when the arm ports (RNG-order contract S1-A-10).
- King-slime spawn gate (census ask): lives **outside** the arm and is ported —
  `DoDeathEvents_AdvanceSlimeRain` (`:79080-79096`) → `world_sim/npcs.rs:3441-3468`
  (150/75 threshold, `-num/2` reset, `!AnyNPCs(50)` gate all match). Two deviations:
  the kill filter counts only `NpcKind::Slime` (`:3453`) where the C# tests
  `Main.slimeRainNPC[type]` (the whole rain roster — the port's other 9 slime kinds
  never count), and the placement is an invented `player_tile_x + 12`
  (`:3460`) instead of `SpawnOnPlayer`'s scan. The rain sky-spawner itself
  (`SlimeRainSpawns` `:5892-5960` → `world_sim/npcs.rs:4345-4400`) is a faithful
  cited port (draw order `Next(45+450*nearby/15)` → X → Y → kind rolls).

**Verdict: WRONG.** Fix wave = port `AI_001_Slimes` whole as a style-1 family module
(the S1-D-3 pattern), with the item-content init as its own slice (RNG-heavy).

---

## 2. Demon Eye family — WRONG (lens drops FAITHFUL)

### 2a. The movement core is a stand-in
- Rust `npc.rs:6740-6757`: normalized-aim accel `dx/d*0.12` clamped ±3 on both axes,
  plus an **invented position bob** `sin(anim*3)*0.4` added to `ny` (`:6751-6752`).
- C# default arm `:52639-52708`: **axis-separated, direction-driven** —
  `num2 = 4*(1+(1-scale))`, `num3 = 1.5*(1+(1-scale))` (`:52641-52644`); X: `-0.1`
  toward `direction`, extra `-0.1` over-cap, `+0.05` wrong-sign brake, hard clamp
  ±num2; Y: `-0.04` toward `directionY`, extra `-0.05` over-cap, `+0.03` wrong-sign,
  clamp ±num3. No normalization, no sqrt, no bob anywhere in the arm.
  `FlyerParams::FLOATING_EYE { accel: 0.12, max_speed: 3.0 }` (`npc_ai.rs:59-62`)
  pins the invented pair with a comment claiming it is `AI_002`'s — **the citation
  is false**; 0.12/3.0 appear nowhere in `:52274-52731`.
- Bounce: Rust `vx = -vx*0.5` / `vy = -vy*0.5` on collision (`:6749`, `:6756`).
  C# `:52283-52306`: `oldVelocity.X * -0.5` (**old**, not current) with the
  direction-gated **minimum kicks** — `|vx|` floored to 2 away from the wall,
  `|vy|` floored to 1. The kicks are what keeps eyes from sticking to ceilings;
  missing entirely.

### 2b. Dawn flee missing (the census ask)
- C# `:52308-52325`: `DespawnEncouragement_AIStyle2_FloatingEye_IsDiscouraged`
  (`:52752-52763`: day && `y <= worldSurface*16` && !graveyard, for types
  2/133/190-194) → `EncourageDespawn(10)`, `directionY = -1`, the two direction
  writes (the C#'s own dead-store quirk at `:52313-52316` — port verbatim). Else
  `TargetClosest()`.
- Rust: nothing — eyes chase forever in daylight. The zombie arm has its gate
  (`npc.rs:6807-6810`); the eye arm never got one. `encourage_despawn` exists
  (`npc.rs:1855-1859`) and is unused here.

### 2c. Missing tails and per-type arms
- Wet tail `:52718-52730` (×0.95 falling, -0.5/tick, cap -4, `TargetClosest`) — absent.
- **WanderingEye (133)** has its own half-life arm `:52502-52637` (hp < 50% →
  6/4 caps + 0.1/0.1 accel, else 4/1.5 + 0.1/0.04) — the port routes it through the
  same stand-in (`npc.rs:3755`), so the enrage never happens.
- Dust `Next(40)` gate `:52710-52717` (types 2/133/190-194) — cosmetic skip, but an
  every-tick RNG draw that must be counted at port time.
- Targeting: `facing = sign(dx)` every tick (`:6758`) instead of `TargetClosest`
  (`npc_target.rs` exists and is used by the zombie — the eye arm never migrated).
  The `sprite_dir` dead-centre keep (`:6764-6768`) cites aiStyle-13 lines
  (`:22740-22752`) — wrong arm citation for a style-2 kind.

### 2d. Lens drops — FAITHFUL
`loot_db_gen.rs:540` `register_to_npc(2, chain_on_failed_roll(common(236,100,1,1),
common(38,3,1,1)))` = Black Lens (236) 1/100 else Lens (38) 1/3, plus the
`:555` demon-eye-banner condition rule — matches the C# registrations. Generated
table, no drift.

**Verdict: WRONG.** Fix wave = port `AI_002_FloatingEye` whole (default arm + 133 arm
+ flee + wet tail + oldVelocity kicks); the 170/171/180 phasing arm can wait on kinds.

---

## 3. Fighter base (zombie family) — rest of arm WRONG; migrated cores FAITHFUL

Already migrated and verified faithful this round:
- **Target core**: `npc_target::target_closest` under the
  `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged` gate
  (`npc.rs:6790-6815` vs `:57336`/`:60300-60330`; the zombie-relevant terms match,
  the unported event terms are commented).
- **Speed ladder**: `npc_ai3_speeds.rs:536-567` ports the `num107` default arm
  (`:58322-58401`) verbatim — per-type caps (186→1.1, 187→0.9, 188→1.2, 189→0.8,
  132→0.95, 200→0.87, 223→1.05), the 489 distance row, the scale row
  `:58374-58377`, 0.07 accel, grounded ×0.8 brake. The kind→C#-id seam
  (`cs_id`, `npc/data.rs:2499-2564`) resolves the variants through the bestiary
  net ids (verified: DoctorBones 52, TheGroom 53, BaldZombie 132 — `bestiary.rs:142-160`).

The REST of the arm:

### 3a. Door bash — MISSING (whole block)
C# `:59990-60087`: touching a door (tile 10) / tall gate (388) increments `ai[2]`;
at 60 ticks the bash fires — `vx = 0.5*(-direction)` recoil, `ai[1] += 5` (gate: 2),
`KillTile(fail:true)` shake, at `ai[1] >= 10` → `WorldGen.OpenDoor`. The blood-moon
gate `flag27` (`:60007`: `(!bloodMoon || getGoodWorld) && !graveyardRoll & flag24`)
**resets `ai[1] = 0` on normal nights** (`:60012-60015`) — i.e. only blood-moon
zombies (and the unbreakable-wall case `flag26`, +6/bash) ever open doors. Rust: no
door interaction anywhere in `update_zombie`; `ai[1]`/`ai[2]` unused for this. The
port has no `Main.bloodMoon` world flag yet — the fix needs that flag first.

### 3b. Blood zombie variants — MISSING kind
`NPCID.BloodZombie` 489 / Drippler 490 have no `NpcKind`, no dispatch. The ladder's
489 row (`npc_ai3_speeds.rs:549-559`) is already ported and waiting. The 489
distance-scaled cap `(2.5 - min(dist*0.0025, 1.5)) * 0.8` (`:58365-58373`) is the
variant's signature behaviour; spawn wiring (`GetZombieSettings` blood-moon arm)
is the spawner's territory (`spawner.rs:915` ports the classic arm only).

### 3c. Step-up / jump ladder — invented stand-in
- Rust `npc.rs:6838-6847`: any x-collision while `on_ground` → `vy = -6.2` (invented
  constant; the C# never uses 6.2), else `vx = 0`.
- C# `:60096-60203`: a **tile-probe ladder** gated on moving toward the sprite
  direction — `height >= 32 && solid(x, y-2)`: `solid(x, y-3)` → **-8** else **-7**;
  `solid(x, y-1)` → **-6**; sunk >20 px into a non-sloped solid → **-5**;
  `directionY < 0` + no floor ahead → **-8** and `vx *= 1.5` (the gap leap). Plus the
  expert above-player hop `-7.9` with the 6-tile platform scan (`:60149-60202`).
- The stand-in is acknowledged (`npc_ai.rs:242-244` "ANALYSIS deferral") — so the
  *existence* is DEVIATION-DOCUMENTED, but the -6.2 value and the missing probes are
  still defects: zombies jump 1-tile walls with the wrong impulse and never clear
  2-tile walls (C# -7/-8) or leap gaps.

### 3d. Daylight burn — INVENTED MECHANIC
Rust `npc.rs:6864-6872`: day → `burn += 1/60`, 6 hp per second. **The C# has no
daylight damage for fighters.** The dawn behaviour is `EncourageDespawn(10)` when
day && above surface (`:57377-57380`, inside the discouraged branch) — a despawn
timer, not damage. The port's discouraged branch only does `vx *= 0.9`
(`npc.rs:6833-6835`). Also missing from that branch: the idle turn-around
(`:57381-57397`: grounded + `vx==0` → `ai[0]++`, at 2 flip direction) and the
`directionY` refresh (`:57370-57373`). Fix: delete `burn`, port the despawn branch.

### 3e. Gravity
Same invented pair as the slimes: `GRAVITY 0.4 / MAX_FALL 8.0` (`npc.rs:6836`) vs
C# 0.3 depth-scaled / 10 (`:90880-90936`); no wet/honey/shimmer merges for zombies.
The ladder's own comment (`npc_ai.rs:208-209`) admits the stand-in but the value
differs from the C# default it cites.

**Verdict: WRONG (rest of arm)** — fix wave order: dawn-despawn + burn deletion
(smallest, highest exposure), jump ladder, door bash (needs bloodMoon), 489/490 kinds.

---

## 4. King Slime — DEVIATION-DOCUMENTED core, four WRONG defects

The arm's structure is a genuine cited port: init latch (`npc.rs:7302-7308` vs
`:43134-43147`), 3000-px despawn setup (`:7311-7317` vs `:43149-43173`), 300-tick
teleport trigger + anti-cheese (`:7320-7333` vs `:43175-43196`), LoS counters
(`:7335-7345` vs `:43198-43213`), shrink 60 / grow 30 ramps (`:7350-7372` vs
`:43221-43282`), hop ladder with the five hp tiers (`:7375-7417` vs `:43284-43344`),
air steer 3/0.2/0.93 (`:7419-7428` vs `:43346-43363`), `num9` scale + bottom-anchor
recentre (`:7432-7442` vs `:43372-43382`), 5% batch gate + `rand(1,4)`
(`:7445-7452` vs `:43387-43393`). The `timeLeft` gates are documented out
(`npc.rs:7346-7349`). Defects:

### K-1. Never-despawn gate bug (missing `ai[1] != 5` guard) — WRONG
C# `:43164`: the far-player state write is gated `if (Main.netMode != 1 &&
this.ai[1] != 5f)`. Rust `npc.rs:7312-7317` writes `ai[0] = 0; ai[1] = 5`
**unconditionally every tick** while the player is dead/far. The shrink branch then
increments `ai[0]` to 1 each tick and it is re-zeroed next tick — `ai[0] >= 60` never
fires, the corner teleport never happens, and (with no `EncourageDespawn(10)` either,
`:43155`) the King Slime **never despawns when the player runs away**. Fix: add the
`ai[1] != 5.0` guard + `encourage_despawn(10)` + the away-facing write
(`:43156-43163`).

### K-2. Minion batch resolution — WRONG (RNG count + invented cap + placement)
C# `:43394-43413` per minion: position `Next(width-32)` + `Next(height-32)` inside
the body, expert `Next(4)==0` → SpikedSlime 535, velocity `Next(-15,16)*0.1` /
`Next(-30,1)*0.1`, `ai[0] = -1000*Next(3)`, `ai[1] = -1` — **5 draws per minion**
(classic) after the one `Next(1,4)`. Rust drain `world_sim/npcs.rs:4290-4302`:
`spawn_slime` at fixed tiles `x + facing*(2+k)`, an **invented 8-minion cap**
(`:4293`, the C# has none), zero per-minion draws, no velocity/ai seeding, no expert
arm. The minions spawn outside the body in a row instead of bursting out of it, and
the `UpdateNPCs` stream is 5×count draws short. Fix: port the spawn loop into the
drain with the draw order above (S1-A-10 contract).

### K-3. Teleport-spot finder — missing gates + invented constant
`king_slime_find_teleport_spot` (`npc.rs:1772-1827`) vs `:43056-43119`:
- **lava gate missing**: C# `:43066` `!tile.lava()` — the port lets the King teleport
  onto lava-covered platforms.
- **cache cap missing**: C# `kingSlimePointCacheSizeMax = 50` (`:95273`, checked
  `:43063`) — the port's `Vec` is unbounded, so the `Next(0, size)` draw range (and
  thus the picked spot) differs whenever >50 candidates exist. Strict-bounds class.
- **invented -46**: the sight-line origin is `y*16 - 46.0` (`npc.rs:1811`) where the
  C# uses `j*16 - base.height/2` (`:43065`) — height is `92*scale` and shrinks with
  hp, so 46 is only correct at full scale.
- The `below_solid` probe (`npc.rs:1804-1806`) approximates `WorldGen.SolidTile`
  (no actuated/halfbrick handling) — minor, note at fix time.
- Band ranges/inclusive bounds and the corner double-count match the C# — faithful.

### K-4. Missing TargetClosest + small folds
- The C# re-aims at the hop launch (`:43317`), at init (`:43145`) and at grow-end
  (`:43268`); the Rust arm **never writes `facing`** — the King hops in whatever
  direction it spawned with until another system turns it. Fix: route the three
  sites through `npc_target::target_closest` (the zombie precedent).
- `hide`/`dontTakeDamage` on the teleport tick (`flag3`, `:43283`) — absent.
- The `life <= 0` early return before the scale write (`:43368-43371`) — absent
  (the port scales a corpse; harmless today, port it anyway).
- `(int)(98*scale)` / `(int)(92*scale)` truncation (`:43378-43379`) — the port keeps
  f32 (`npc.rs:7434-7435`); int-vs-float class, sub-pixel.
- `localAI[3]` is a one-shot `flag` in C# (`:43138-43141`); the port's
  `king_slime_local[3] == 1.0` stays true forever (`npc.rs:7433`) — benign while
  `num9 == scale` makes the recentre a no-op, but the semantics differ.
- Slime-rain spawn gate: see §1c — ported, two deviations (kind filter, placement).

**Verdict: DEVIATION-DOCUMENTED core / WRONG defects K-1..K-3** (K-4 minor). K-1 is
the priority: it is a visible boss-fight escape exploit in every session.

---

## Fix-wave seeds (priority order)

| Seed | Scope | Size | Depends |
|------|-------|------|---------|
| A1-1 | Zombie dawn: delete `burn`, port `EncourageDespawn(10)` + idle-turn branch (`:57375-57401`) | S | — |
| A1-2 | KingSlime K-1 despawn guard + K-4 TargetClosest sites | S | npc_target (landed) |
| A1-3 | Eye family whole-arm port (`AI_002` default + 133 + flee + wet + kicks) | M | npc_target |
| A1-4 | Slime family whole-arm port (`AI_001` machine, per-type rows) | L | — |
| A1-5 | Slime item-content init (`:60356-60575` + `:61734-61848`, RNG-heavy) | M | A1-4 |
| A1-6 | KingSlime K-2 minion draws + K-3 finder gates | S | — |
| A1-7 | Fighter jump ladder (`:60096-60203`) + door bash (`:59990-60087`) | M | bloodMoon world flag |
| A1-8 | Gravity rebase: 0.3 depth-scaled / 10 + honey/shimmer merges (shared with S1-M-1) | M | S1-M-1 |
| A1-9 | BloodZombie 489 / Drippler 490 kinds + spawn wiring | M | A1-7, bloodMoon |

RNG note (S1-A-10): none of the four live arms draws `Main.rand` today except the
KingSlime batch `Next(1,4)` and the teleport-spot pick — the C# arms' draws (slime
content init, eye/fighter dust gates, king minion bursts) are all **deficits**, so
the fix waves add draws; each must land with a stream-replay witness.

*Audit performed 2026-11-19, code-only mode: no cargo/rustc/rustfmt/tests/dotnet/
graphify invoked; evidence = file reads + git only.*
