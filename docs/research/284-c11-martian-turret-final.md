# 284 — W4-c11 RETIRE/SEAL: the aiStyle-73 Martian Turret FINAL + the martian-turret-family bible

Status: SEALED (W4-c11 retire session, 2026-11-21). Code-only — no
cargo/rustc/tests/rustfmt/graphify invoked; every witness WRITTEN NOT RUN.
Registry number: **284** = max-on-tips (283, f4 `283-npc-dust-final.md`) + 1,
per research/150's "281+ FREE" guidance re-probed at THIS session's fetch
(all remote tips `git ls-tree docs/research/`; 281 is now 8-way collided —
d9/g48/c25/d30/f39/d125/c9/d106 all took it after the registry published;
282 is 3-way d125/g48/f39; 283 single-claimant f4; **284 single claimant,
this doc**). C# cites are `decompiled/Terraria/{NPC,Projectile,ID/*}.cs`
(stable), read in ≤100-line windows.

Branch: `task/W4-c11-style73-martian-turret`, base `a725acb04` (wave-3
magic+summon land), pre-retire tip `00a42d849`. The primary checkout was
never touched; all sibling lanes probed READ-ONLY via `git show`/`git grep`.

## 1. git-log census — what landed on style73-martian-turret

| commit | phase | content |
| ------ | ----- | ------- |
| `be1d00d95` | S1-D-3 (pre-branch, on gate/lc-merge `3abac7ac3`) | the UNWIRED pure module `npc_ai_style73.rs` (the style73 precedent every later unwired round cites): `style73_step` verbatim `NPC.cs:35102-35217`, `MARTIAN_TURRET` defaults row `:13922-13934`, `turret_find_frame` `:71663-71696`, `turret_materialize_dust_plan` (count-only), 14 witnesses |
| `a9b22f059` | W4-c11 phase 1 | the 435 MartianTurretBolt `proj_data` row (`projectile/data.rs`: 10×10 aiStyle 1 hostile, `ignoreWater` overlay `Projectile.cs:5062`, `projFrames` 4 `Main.cs:8532`, stock penetrate 1/tileCollide/timeLeft 3600) + the SetDefaults-slice witness |
| `6940d6ee7` | W4-c11 phase 2 | the axis claim (`npc.rs` aiStyle 73 arm) + `ai_073_martian_turret` adapter (session candidate, netMode 0, classic 1.0, one-tick `just_hit`; the 435 shot queues on `wants_hostile_shot`; `body_gravity` + blocked-axis collision tail, the style-72/spore folds); the hold-still kind arm retires to `unreachable!`; FindFrame case 387 delegates to `turret_find_frame` (BlazingWheel precedent); 2 wiring witnesses |
| `00a42d849` | W4-c11 phase 3 | the draw-side light feed `turret_light` (`app/frame/draw.rs` beside the ForceBubble arm — ai[1]/60 fade `:35167`, constant (0.1,0.5,0.7) `:35175`, each quotient truncated); ANALYSIS wiring record + the dust deferral note; light witness + 3 anchors rows |
| (this retire) | W4-c11 retire | the ring ACTIVATION (§3): `turret_materialize_ring` adopted from e2 verbatim + the adapter drains the plan into the live pool on `UpdateNPCs`; 3 witnesses (2 module + 1 wiring); this doc + ANALYSIS/PORT_MAP/ROADMAP/anchors updates |

The pre-retire diff `a725acb04..00a42d849` touches exactly 6 files
(anchors.txt, app/frame/draw.rs, npc.rs, npc_ai_style73.rs,
projectile/data.rs, docs/ANALYSIS.md) — task-scoped, verified.

## 2. All-arms-wired / kinds-rowed census + remaining gaps

**The style-73 axis is CLOSED.** The whole-tree `this.aiStyle = 73;` census
returns exactly two hits: `NPC.cs:13926` (the type-387 SetDefaults branch)
and `Projectile.cs:4955` (the fountain projectiles 419-422 — projectile
space, out of NPC scope; the wiring engine's dropped fountain requests ride
it). No dynamic reassignment to 73 exists. The ONE NPC kind (MartianTurret
387) is rowed (`NpcKind::MartianTurret`, wire id 214, `npc/data.rs`:
ai_style 73, size 20×50, hp 200, dmg 10, def 40, sounds Hit_53/Killed_56)
and wired (axis arm → `style73_step` → adapter tail). The kind-match arm is
`unreachable!` (the retirement guard witness). FindFrame 387 delegates.
The light feed is live. **Zero unwired style-73 arms remain.**

**The 435 projectile row is rowed HERE, activated by composition:**

| 435 surface | owner lane | tip | state at that tip | after c11 merges |
| ----------- | ---------- | --- | ----------------- | ---------------- |
| `proj_data` row + `projFrames` 4 | **c11** `a9b22f059` | this branch | LIVE (spawn drain resolves it) | — |
| AI_001 overlay (`Projectile.cs:54909-54933`: the ai[1] Item12 latch, fade-40, spriteDirection, 3-tick 4-frame machine, `:57642-57648` direction-split ToRotation tail; `flag3=false` `:55992` straight flight) | f20 slice E `be6ca78c6`, module `projectile_ai001_e.rs` | `0883a80bc` | ROW-GATED INERT (`proj_data(435)` is None on f20; the dispatch's `else if let Some(data)` guard never routes) | ACTIVATES with NO further edit — the census verified no other branch ports the overlay (26f); the module's private `MARTIAN_TURRET_BOLT` const dedupes onto `data.rs`'s at merge (cosmetic, both = 435) |
| Kill death dust (`:73467-73478`: Next(5,10) dust-226 center-lerp burst) | g65 `projectile_death_dust.rs:977` (`center_lerp_burst(…, 5, 10, 226, 0.5, 1.6, false)`) | `f6e07bb90` | kind-keyed on the DeathSnapshot — inert until a 435 slot can die | ACTIVATES (the row lets bolts spawn; the drain's pass-tail hook fires on their Kill) — §4 |
| AddLight `:54932` (0.3, 0.8, 1.1) | e6 second wave `proj_ai_lights.rs` ("no row. Absent."), census frozen in f7 `tests/proj_addlight_census.rs:156` | `c8b072b69` / `67110ccfb` | ABSENT wholesale (no row) | the row lands but the e6 arm is a documented-absent NOTE, not a kind-gated port — **GAP: the 435 light needs the e6/c19 lane's row-absence re-probe** (their "no row → absent" convention means the arm must be ADDED, not auto-activated; owner: the lights family, c19 tip `5400e0374` sealed 1-67 + e6 68-107) |
| per-tick trail dust | g63/g64 | — | NO 435 site (the C# AI_001 435 block has no dust — only the Kill burst) | nothing owed |
| Item12 spawn cue | f20 slice E (`wants_sound` seam) | `0883a80bc` | inert with the overlay | activates with it |

**Remaining gaps (family-wide, NOT style-73 arms):**

1. **The Engineer's turret-placement arm** (`NPC.cs:58984-59039`,
   AI_003_Fighters' type-386 block: the ai[2] cycle + the
   `CountNPCS(387) >= 4*CountNPCS(386)` cap `:58996`, the justHit −30
   `:59000-59003`, the `ai[2]==30 && netMode!=1` placement `:59004` — the
   100-attempt ±5/±2 tile scan with the lava/solid/`SolidTiles(-1..1,-4..-1)`
   gates, `NewNPC(num157*16 - width/2, num158*16, 387)` + the
   `position.Y = num158*16 - height` feet plant `:59030-59031`). The port's
   Engineer rides `update_skeleton` (the aiStyle-3 walker fold); the
   placement block is UNPORTED — turrets only enter via the netglue seed
   ctor (`app/netglue.rs:3505`, `spawn_cave_walker` — the feet plant) and
   the invasion spawn table. Owner: the invasion/AI_003-shooter wave (the
   kind arm's comment cites it). The two placement draws per attempt
   (`Next(px-5,px+5)` then the Y-loop seed `Next(py-5,py+5)`, `:59018-59019`)
   ride the caller's stream when ported.
2. **The 435 AddLight** (the table above) — the lights-family re-probe.
3. **The MartianWalker 520's 592 laser** — g4's FINAL fighter census
   (`9912498fc`-era research/91, `69d7737ba`) already pins it: walk routed,
   592 unported (projectile-work lane). Not a turret-family arm; listed
   for the bible's completeness.
4. **The turret's HitEffect/death visuals** — owned by g62 (every-strike
   ×20 dust-226, `npc_hit_dust.rs:1262` the 384|387|520 arm, tip
   `f5ce54743`) and f10 (death 30-dust-226 burst + gores 603/603/604,
   `npc_hit_effect_c.rs:3848-3849`, sites 451-453, tip `afea4c0fc`); both
   kind-keyed, activate when their lanes merge. Sounds Hit_53/Killed_56
   rowed (e9/d80 lanes carry the audio seams).

## 3. e2c-MartianTurret-35157/35162-STAGED-gap-sites — ACTIVATED

The owed-pool item `e2c-MartianTurret-35157/35162-STAGED-gap-sites` is
**DISCHARGED by this retire**. History: e2 batch 6 (`9912498fc`,
2026-11-21T10:25:57+02:00) staged the gap-window emitter
`npc_dust.rs::turret_materialize_ring` (e2 tip `bfc081305`, file ~line
1030) with the doc "STAGED — the style-73 machine is unwired (the kind
arm is a stand-in hold)". That precondition was discharged by c11 phase
2 (`6940d6ee7`, pushed 2026-11-20 — e2's note was stale at staging time);
the ring stayed count-only in the adapter pending the wiring-wave A/B
(ANALYSIS dust-deferral note, `00a42d849`).

**The activation (this retire):**

- `npc_ai_style73.rs::turret_materialize_ring` — the e2 body adopted
  VERBATIM (keep-one): for each of the plan's 8 placements, ONE
  `DustPool::new_dust(rng, ctx, world, Center, 0, 0, 226, 0, 0, 100,
  default, 0.5)` then post `noGravity = true`, `position = placement`,
  `velocity = Zero` (`NPC.cs:35157-35165`). The signature generalizes
  `&mut UnifiedRandom` → `&mut R: DustRng` (the pool's own trait bound —
  zero behaviour change; `UnifiedRandom` implements `DustRng`).
- `npc.rs::ai_073_martian_turret` now takes `dust`/`dust_ctx` (threaded
  from `update_rng_dust`, the Everscream S1-F-13c precedent — the live
  sim already passes `app.dust` + `app.spawn_ctx()` at
  `world_sim/npcs.rs:4467`); on fade ticks it drains
  `out.materialize_dust` into the ring AFTER the pure arm returns,
  BEFORE the gravity tail. The pure `style73_step` is UNCHANGED (the
  plan seam stays count-only; its 14 witnesses stand).
- **Stream order**: on a fade tick the arm's early `return` (`:35168`)
  precedes the shot cycle, so the ONLY draws are the ring's — 8 ×
  `NewDust` internals in C# pair order on the caller's `UpdateNPCs`
  stream (`Main.cs:17767-17769`'s SwapRandom wraps UpdateNPC, dust
  included). Per dust on an empty pool: 6 draws (pos X/Y `Next(1)` —
  the 0×0 box clamps 5×5, vel X/Y `Next(-20,21)`, frame Y `Next(3)`,
  scale `Next(-20,21)`; `Dust.cs:153-193`); type 226 matches NO
  tweak-chain arm and is NOT dangerous dust → 48 draws/fade tick, plus
  the pressure-band rolls when the pool fills (the verbatim pool owns
  them). The witness `the_ring_consumes_six_pool_draws_per_dust` pins
  the 48-draw census against a mirrored oracle; the frame fields carry
  the ≥100 loop's two passes (226→126→26: frame_x 2260−2000=260,
  frame_y 10·Next(3)+60).
- **Merger keep-one**: the e2 `npc_dust.rs::turret_materialize_ring` +
  its witness `w4_e2_turret_materialize_ring_eight_placements` DROP at
  merge (this module's copy is the same body; the e2 witness's mirror
  shape is re-derived here as `the_ring_draws_eight_placements_in_csharp_order`).
  The e2 STAGED-doc note flips to ACTIVATED-BY-C11. No other lane claims
  the ring (the g61 spawn-visuals lane has no 387 site — verified).
- **Witnesses (WRITTEN NOT RUN)**: module
  `the_ring_draws_eight_placements_in_csharp_order` (the e2 mirror
  replay, 8 slots, type/noGravity/alpha/velocity/position/scale pins),
  module `the_ring_consumes_six_pool_draws_per_dust` (the 48-draw
  oracle census + frame_x 260/frame_y +60), wiring
  `npc.rs::w4_c11_the_fade_ring_draws_on_the_live_pool` (one
  `update_rng_dust` fade tick spawns exactly 8 on the caller's stream,
  the twin-pool replay matches, the last fade tick still rings, the
  latch tick spawns NOTHING and the arm draws nothing else).

## 4. g65-Kill-309 — compose

The owed-pool item `g65-Kill-309` refers to g65's final accounting
(`b0842907c`, ANALYSIS "Census completeness"): the whole Kill body
(`Projectile.cs:70642-77262`, 406 Dust.NewDust*/CloneDust sites)
cross-referenced against the drain's **309 distinct kind arms** (the
commit-message count; the later keep-one receipt `f6e07bb90` /
research/26i re-counts **317 distinct kinds incl. the 121..=126 /
411..=414 range arms** — 309 literal match arms + 8 range-covered kinds;
both counts describe the SAME drain, no disagreement in coverage).
The ledger: 393/406 sites covered by the drain, 6 other-lane-claimed
(442→i16, 608×5→d12), 7 documented-deferred (1084/949 TorchID+Disco,
868/869/996 WorldGen randomizers, 655 wet-item/NPC-spawn, 91/92
star-spawn, 89 shard-split, 183 bee-burst) — **0 unclaimed** (26i §1).

**The c11 composition is the 435 arm** (`projectile_death_dust.rs:975-977`
at g65 tip `f6e07bb90`):

```
435 => center_lerp_burst(s, dust, rng, ctx, world, 5, 10, 226, 0.5, 1.6, false)
```

verbatim `Projectile.cs:73467-73478` (re-read this session): the
`Next(5,10)` count draw BEFORE the spawns; per dust
`NewDust(Center, 0, 0, 226, 0, 0, 100, default, 0.5)`, post
`velocity *= 1.6`, `velocity.Y--`, `position = Lerp(position, Center,
0.5)`, `noGravity = true`. The shared `center_lerp_burst` helper
(`:397-423`) serves 435/436/462/631 (the 444/732 variants inline).

- **No c11-side code owed**: the drain is kind-keyed on the
  DeathSnapshot; c11's `proj_data` row is the ONLY missing precondition
  (g65's branch has no 435 row — verified `git grep`). At merge the row
  lands, bolts spawn (the axis shot → `wants_hostile_shot` → the sim's
  spawn drain), their Kill snapshots reach the drain
  (`world_sim/projectiles.rs:419-425` at g65 tip — the pass-tail drain
  on the `UpdateProjectiles` stream), and the burst fires. The g65
  witness `w4_g65_the_center_lerp_bursts` (`:5607`) already pins 435
  (count ∈ [5,9], type 226, noGravity) with a hand-built snapshot.
- **Stream compose**: the ring (§3) draws on `UpdateNPCs`; the Kill
  burst draws on `UpdateProjectiles` — DIFFERENT streams, no ordering
  interaction. Within `UpdateNPCs` the ring's 48 draws/fade tick now
  precede every later NPC's draws (the deliberate activation shift —
  the deferral note's A/B is THIS retire's witness set; the pure-arm
  witnesses are unchanged because the arm itself still draws nothing).
- **Keep-one**: no duplicate 435 Kill body exists on any probed lane
  (d78's EVP trio is 452/454/448; g3's census calls drop per-kind as
  g65 bodies push — 26i §2/§3; the 435 arm is g65-UNIQUE). c11 ports
  NO Kill code; this section is the compose receipt.

## 5. The martian-turret-family bible (FINAL)

One paragraph per surface; ONE owner per body; tips pinned at this
session's fetch.

**The machine (aiStyle 73, NPC.cs:35102-35217)** — c11 owns the whole
arm: `npc_ai_style73.rs::style73_step` (the pure C#-order port —
TargetClosest(faceTarget:false), spriteDirection mirror, ×0.93 X decay +
strict ±0.1 snap, the flag74 1920×1200 firing rect, the 120-tick
materialize fade with the ai[1]-first increment/(int) alpha
ramp/dontTakeDamage/frame pin/early return, the one-shot 120→121
netUpdate latch, the shot cycle's −120 hold/justHit −30 reset/the
ai[0]==60 four-draw jittered Normalize+NaN→−UnitY ×14 aim at
GetAttackDamage_ForProjectiles(35,28) — the 381..=392 Martian 0.8 band),
the adapter `npc.rs::ai_073_martian_turret` (session candidate/netMode
0/classic/one-tick just_hit; the shot queues on `wants_hostile_shot`;
body_gravity + the blocked-axis collision tail), FindFrame 387
(`turret_find_frame`, the 1/+20/+40 tiers, 10/15-tick flips, 2-row
strip), the defaults audit row (`MARTIAN_TURRET`). SEALED — no other
lane touches the arm (the d30 band 65-72 retire `ec89521fb` stops at 72;
the f20/d78/g65 projectile lanes never enter NPC space).

**The ring (NPC.cs:35131-35166, sites 35157/35162)** — c11 owns the
geometry (`turret_materialize_dust_plan`) AND the draw
(`turret_materialize_ring`, the e2 body adopted verbatim — §3). e2's
staged copy DROPS at merge. The 8 placements/type-226/alpha-100/
scale-0.5/noGravity/position-overwrite/velocity-zero contract is pinned
by both lanes' witnesses (same derivation).

**The light (NPC.cs:35167/:35175)** — c11 owns the NPC-side feed
(`turret_light` in `app/frame/draw.rs`, the ForceBubble precedent: the
sim's outcome light is discarded, the feed recomputes from ai[1] +
centre; fade ×ai[1]/60 below 120, constant (0.1,0.5,0.7) at/above;
each quotient truncated). The c20 addlight-npcs lane has no 387
duplicate (the feed predates it; keep c11's).

**The bolt (435 MartianTurretBolt)** — FOUR owners, disjoint surfaces:
c11 the `proj_data` row + projFrames (phase 1); f20 slice E the AI_001
overlay (`projectile_ai001_e.rs`, row-gated inert → activates at merge,
NO dedupe owed — 26f census); g65 the Kill dust burst (§4); e6/f7 the
AddLight census slot `:54932` — currently documented-ABSENT ("no row"),
the ONE remaining activation gap (§2 item 2, owner: the lights family's
row-absence re-probe). The Item12 cue rides f20's `wants_sound`. No
trail dust exists in the C# 435 block (g63/g64 verified empty).

**The defaults/roster (S1-F-13d)** — the 17-kind martian roster is
rowed on main (`npc/data.rs`: Officer 383 wire 68, GrayGrunt 385/69,
Engineer 386/70, GigaZapper 389/71, Walker 520/72, Probe 399/187,
Drone 388/188, Scrambler 381/85, RayGunner 382/86, ScutlixRider 390/87,
Scutlix 391/88, Saucer 392/89, SaucerTurret 393/90, SaucerCannon 394/91,
SaucerCore 395/92, ForceBubble 384/213, MartianTurret 387/214); the
turret's sounds (Hit_53/Killed_56), buff immunities (npc_buffs — the
whip-immune FALSE pin), bestiary entry, NPC_387 sheet, invasion group 4
+ the 0-point kill gate (`world_sim/npcs.rs:2500/:3252`) all landed
pre-c11. The AI wiring map: styles 72/73/75 axis-claimed (ForceBubble/
Turret/Rider+Saucer mounts), 74/76/80 kind machines (Drone/SaucerCore/
Probe), the aiStyle-3 walkers fold (Grunt/Engineer/Zapper/Walker +
Officer's bubble block + Scrambler/RayGunner shooter block + Scutlix
carrier). The Engineer's turret-PLACEMENT block (§2 item 1) is the
family's one unported NPC-side arm — invasion-wave owned, not c11's.

**The death/hit visuals** — g62 every-strike (×20 dust-226 at
(-dir,-1), Centre-Lerp 0.25, scale 0.5 — the 384|387|520 shared arm,
`npc_hit_dust.rs:1262`, tip `f5ce54743`), f10 death (30-dust-226 burst
SpeedX −1.5·dir + gores 603/603/604, `npc_hit_effect_c.rs` sites
451-453, tip `afea4c0fc`, the batch-45 commit `b0b74c7e7`), g60/f5 the
gore-dispatch keep-one matrix (research/110 `3b5f9b3cc` — the f10-owned
martian arms are claimed, no unclaimed gore). All kind-keyed; activate
at their merges; c11 ports none.

**The spawn** — the netglue seed ctor (`app/netglue.rs:3500-3505`,
`spawn_cave_walker` — the C# feet plant `position.Y = num158*16 -
height`) + the invasion table (`start_invasion_in`, the martian size
ladder). The Engineer's in-game placement (§2 item 1) is the gap.

**Gate command** (for the merger, NOT run here):
`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
— judge by the `test result: ok` lines.

## 6. Verification record

Code-only session: no cargo/tests/rustfmt/graphify invoked (the task
contract). Every C# claim re-read at the cited lines this session
(NPC.cs:35102-35181, :58950-59039, Projectile.cs:54905-54944,
:73460-73489, particles.rs:435-628 — ≤100-line windows). Every sibling
tip pinned by `git rev-parse` after `git fetch origin --prune`. The
witnesses are WRITTEN NOT RUN; their expected values were hand-derived
(the 6-draw-per-dust census from the verbatim pool body, the frame
260/+60 from the ≥100 loop, the ring mirror from the e2 derivation).