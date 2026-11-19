# 55 — Worm SIM-PASS fidelity audit (census slice A-2, distrust-verbatim)

Audit base: `c7fe175d` (origin/main, 2026-11-19). Worktree `TR-audit5`, branch
`docs/audit-worms`. **Code-only mode: no compiler invoked** — every claim below is
file-read evidence (C# `decompiled/Terraria/NPC.cs` / `WorldGen.cs` vs the Rust
tree at the base commit). Catalogue witness: `f1701eb0` (the aiStyle-29 escape-clause
audit whose defect checklist this slice applies to the worm pass).

Scope: the census §A-2 row — the **14 SIM-PASS worm kinds** (census rows 25-33,
48-49, 141-143: Devourer 7/8/9, GiantWorm 10/11/12, EoW 13/14/15, Leech 117/118,
SolarCrawltipede 412/413/414) plus kind 144 (StardustWormHead 402), audited against
`AI_006_Worms` **whole** (`NPC.cs:50963-52273`). The Destroyer (134, census row 50,
STAND-IN) is **R55's lane (TR-wire14)** — only the shared worm-follow mechanics are
audited here; Destroyer-specific findings are collected in §5 for R55.

The live pass: `crates/tr-app/src/world_sim/npcs.rs::update_worms_in` :1055-1308
(init/chain-spawn :1067-1143, realLife pool :1148-1170, follower trail + orphan
rules :1173-1299, head machine `worm_head_step` :1316-1374). Kind helpers:
`npc/data.rs::is_worm` :809, `is_worm_head` :868, `is_worm_mid` :883, `worm_next`
:897, `worm_tunables` :922, `shares_carrier_life` :838; carrier resolution
`npc.rs::carrier_index` :3419.

## 0. Verdict table

| # | Target | Verdict | Severity |
| --- | --- | --- | --- |
| T1a | Head rotation (`NPC.cs:52201`) | **MISSING** — heads never rotate | HIGH (visual + aim read) |
| T1b | Head air/dig steering model (`:52112-52198`) | **STAND-IN** — instant velocity snap vs the C#'s 3-arm accel steer | HIGH |
| T1c | Budget-roll RNG order (`:51310-51342`) | **DIVERGENT** — discarded `Next(8,13)` draw dropped for 4 families | HIGH (RNG stream) |
| T1d | Surface gate + despawn (`:51110-51148`) | **MISSING** — GiantWorm/Digger/Leech chase above surface forever | MED |
| T1e | Corruption escape + chain despawn (`:52004-52050`) | **MISSING** — Devourer/EoW never flee/despawn outside evil | MED |
| T1f | 1000-px no-player dig override (`:51598-51615`) | **MISSING** | MED |
| T1g | flag3 tile scan shape (`:51525-51597`) | **DIVERGENT** — margins, overlap test, solidTop, liquid-on-inactive, `Next(100)` crumble draw | MED (RNG + state) |
| T1h | Per-family tunables (`:51670-51773`) | **PARTIAL** — Digger/Wyvern/BoneSerpent rows absent (kinds absent), EoW expert tiers absent | MED |
| T2a | Follow model (leader snap, not position history) | **FAITHFUL model** — the C# is a direct leader snap; the port matches | OK (see T2b/c) |
| T2b | Child spawn placement (`NewNPC` Bottom, `:80770`) | **DIVERGENT** — child top at parent bottom vs C# child bottom at parent bottom | MED |
| T2c | Pass order (followers before heads) | **DIVERGENT** — C# slot order runs the head first (children take higher slots) | LOW-MED |
| T2d | Segment spacing overrides (`:51880-51904`) | **PARTIAL** — crawltipede `width+6` missing; wyvern 42 / EoW-FTW 62 ride missing families/seeds | LOW |
| T2e | realLife pool + EoW exception (`:51129-51136`, `:81374-81378`) | **FAITHFUL** (S1-D-6, re-verified) | OK |
| T3a | Wyvern 87-92 family | **UNPORTED** — no kinds, no 14-link chain, no altitude gates | HIGH (coverage) |
| T3b | EoW split on kill (`Transform(13/15)`, `:51511-51518`) | **STAND-IN** — kind swap without Transform's life rescale/reposition/ai-reset | MED |
| T3c | EoW shadow-orb summon (`WorldGen.cs:54294-54321`) | **DIVERGENT** — `downed_eow` gate vs C# `AnyNPCs(13)`; crimson-Brain arm absent | MED |
| T3d | EoW expert content (70 segments, 12/0.15 speed, VileSpit 666, alpha fade) | **MISSING** (expert-mode framework absent; alpha fade is classic too) | MED |
| T3e | Leech tail 119 | **MISSING** — `worm_next(LeechBody) == LeechBody`, chain never grows a tail | MED |
| T3f | StardustWormHead 402 (kind 144) | **ORPHANED** — kind + tunables exist, `is_worm` excludes it; AI_006's 402 arm unported | MED |
| T3g | Crawltipede always-fly + own aim machine (`:51560-51563`, `:51777-51853`) | **STAND-IN** — rides the generic dig/air head instead | MED |
| T4 | Destroyer shared mechanics | audited; Destroyer-specific deltas → §5 (R55) | boundary |

## 1. Head AI per family vs `AI_006_Worms` (T1)

### T1a — head rotation never written (HIGH)

C# `:52201` (inside the no-leader `else`, i.e. every head, every tick):
`this.rotation = (float)Math.Atan2(base.velocity.Y, base.velocity.X) + (float)Math.PI / 2f;`
The port's `worm_head_step` (`world_sim/npcs.rs:1316-1374`) writes **no rotation**;
the only worm rotation write in the pass is the follower's `:1258`
(`dy.atan2(dx) + 1.57`). `npc.rs`'s FindFrame worm arm is empty (`:5941-5949`), and
the renderer rotates only by `n.rotation` (`render/entities.rs:3018-3040`, worm
kinds in the rotate list). Result: **every worm head draws unrotated** (frame-up)
while digging/flying in any direction. Followers rotate correctly.
Fix spec: at the end of `worm_head_step`, `npc.rotation = npc.vy.atan2(npc.vx) +
std::f32::consts::FRAC_PI_2` (the C# head uses `Math.PI/2`, not the followers'
`1.57f` literal — keep the two constants distinct).

### T1b — the dig is an accel steer, not a velocity snap (HIGH)

C# dig branch `:51997-52002` computes the snapped aim and scales it
(`num67 = num45 / num62; num48 *= num67; num49 *= num67`) — but **never assigns it
to velocity**. The velocity then approaches that aim through the three-arm steer
`:52112-52198`:
- same-sign arm `:52114-52153`: ±`num46` per axis toward the aim, plus the two
  cross-brakes (`|num49| < num45*0.2` with X-sign reversal → `vy ± num46*2`, and
  the X mirror `:52143-52153`);
- `|num48| > |num49|` arm `:52155-52176`: X ±`num46*1.1`, and below half speed the
  Y sign-assist ±`num46`;
- else arm `:52177-52198`: the Y mirror.
The port (`:1356-1366`) instead sets `vx = dx; vy = dy` outright — instant
full-speed direction changes, no turn radius, no ramp. This is the W81-class
"invented shortcut" the distrust-verbatim rule targets.
Fix spec: port `:52112-52198` verbatim (three arms, the 0.2/0.5 bands, the 1.1/2×
multipliers, f32 order), feeding the snapped+scaled `num48/num49`; keep the
`flag6`/`flag8` pre-modifiers' positions (T1e, T3a).

### T1c — budget-roll RNG order (HIGH, determinism)

C# `:51310` rolls `ai[2] = Main.rand.Next(8, 13)` for **every** head type in the
`:51303` list, then overrides:
- type 10 (GiantWorm): **another** `Next(4, 7)` `:51313` — two draws;
- type 13 (EoW): `GetEaterOfWorldsSegmentsCount()` `:51317` — no extra draw, but
  the `Next(8,13)` was still consumed;
- type 39: `Next(14,23)` `:51321`; type 95: `Next(6,12)` `:51333`; type 98:
  `Next(20,26)` `:51337`; type 117 (Leech): `Next(3,6)` `:51341` — all two draws;
- type 7 remix ×2 `:51343-51346` (secret seed, out of scope).
The port (`npcs.rs:1090-1105`) draws exactly once per family: Devourer
`next_range(8,13)` ✓, GiantWorm `next_range(4,7)` ✗ (missing the discarded
`Next(8,13)`), Leech `next_range(3,6)` ✗ (same), EoW constant 65 ✗ (missing the
discarded draw entirely). `UnifiedRandom::next_range` is the faithful stream
(`unified_random.rs:173`), so every worm spawn shifts the `UpdateNpcs` stream for
all later consumers.
Fix spec: always draw `next_range(8,13)` first at chain start, then the per-family
override draw (discard the first result where the C# overrides); EoW keeps the
first draw's stream advance and uses 65/70.

### T1d — surface gate + EncourageDespawn (MED)

C# `:51110-51128`: `flag = true` for types 10/39/95/117/510 (GiantWorm,
BoneSerpent, Digger, Leech, TombCrawler); `:51137-51148`: target dead **or**
(`flag` && player above `worldSurface*16`) → `TargetClosest()`,
`EncourageDespawn(300)`, and `velocity.Y += num10` (0.2; 0.1 for 513). The port
has no surface gate and no despawn discouragement anywhere in the pass — a
GiantWorm chases the player into the sky indefinitely.
Fix spec: add the flag table + the gate before the head machine; the despawn
timer rides the framework's `time_left`/EncourageDespawn seam (shared with other
audits); the +0.2 sink applies to the head's vy while gated.

### T1e — corruption escape (MED)

C# `:52004-52050` (dig branch): for types 7/13, if the target is outside
Corrupt/Crimson or dead → `flag6`; then if **no** living player is in
ZoneCorrupt: below `(rockLayer + maxTilesY)/2` the head deactivates **and walks
the `ai[0]` chain deactivating every segment** (`:52028-52047`); the aim is
overridden to straight down (`num48 = 0; num49 = num45`, `:52048-52049`).
The port has none of it — the EoW/Devourer follow the player into any biome.
Fix spec: port flag6 + the chain-despawn walk (the port's `worm_id`/`seg_index`
chain stands in for the `ai[0]` walk) + the downward aim override; needs the
player's zone flags (`scene_metrics.zone_corrupt/zone_crimson` exist).

### T1f — the 1000-px dig override (MED)

C# `:51598-51615`: a head with `flag3 == false` and **no player within a
1000-px half-box** (`num43 = 1000`, rectangle intersect over all 255 players)
forces `flag3 = true` — far-away heads burrow toward the player instead of
stumbling in air. The port's air branch never checks player distance.
Fix spec: after the tile scan, if !flag3 and the player centre is outside the
±1000 box, set flag3 (single-player port: one rect test).

### T1g — flag3 scan shape (MED, RNG + state)

C# `:51525-51597`: scan margins `(int)(x/16)-1 .. (int)((x+w)/16)+2` (Y same),
clamped to the world; per candidate tile the solid rule is
`nactive && (tileSolid || (tileSolidTop && frameY == 0))` **or** `liquid > 64`
(liquid counts on inactive tiles too); a tile counts only when its 16-px box
**actually overlaps** the NPC rect (`:51586`); each overlapping active solid tile
draws `Main.rand.Next(100)` and on 0 fires `KillTile(fail:true, effectOnly:true)`
(`:51590-51593`, skipped for type 117).
The port (`:1326-1338`): no −1/+2 margins, `x0..=x1` inclusive without the
overlap test (any solid in the coarse box flips dig mode), no `tileSolidTop`
arm, liquid gated on `t.active()`, and **no `Next(100)` draw** — another stream
divergence on every digging tick.
Fix spec: port the margins + per-tile overlap + the solidTop/liquid rule; the
crumble draw must run per overlapping solid tile in scan order (the effect-only
KillTile can ride the dust seam, the draw cannot be skipped).

### T1h — tunables (MED)

`worm_tunables` (`data.rs:922-935`) vs C# `:51670-51773`: Devourer 9/0.1 ✓,
GiantWorm 6/0.05 ✓, EoW 10/0.07 classic ✓ — but EoW expert 12/0.15
(`:51696-51700`) and FTW +4/+0.05 (`:51701-51705`) absent; Digger 5.5/0.045
(`:51672-51676`), Wyvern 11/0.25 (`:51720-51724`), BoneSerpent 9/0.1
(`:51764-51773`) absent with their kinds; Leech's WoF-life rage tiers
(`:51745-51763`: +1/+0.1 at <50%, again <25%, +2/+0.1 <10%) absent. The
crawltipede 10/0.3 row exists but belongs to the special machine (T3g).

## 2. Body/tail follow mechanics (T2)

### T2a — the model is a leader snap, NOT position history (verified)

The task brief suspected a position-history chain. **The C# does not use one**:
`AI_006` `:51867-51922` — every linked segment (`ai[1] > 0`) recomputes
`delta = leader.Center - own.Center`, sets
`rotation = Atan2(dy, dx) + 1.57f`, `num62 = (len - num63)/len`, zeroes velocity
and adds `delta * num62` to position — a rigid distance constraint toward the
leader's **current** centre. The port (`npcs.rs:1249-1258`) implements exactly
this model (own width as `num63`, `.max(0.001)` guard on the divide — the C# has
no guard; at len 0 the C# produces NaN, the port a finite overshoot; negligible
but note it). **Verdict: model FAITHFUL.** The deltas are T2b-T2d.

### T2b — child spawn placement (MED)

C# spawns each link with `NewNPC(..., (int)(pos.X + width/2), (int)(pos.Y +
height), ...)` (`:51347`) and `NewNPC` sets `nPC.Bottom = (X, Y)` (`:80770`) —
the child's **bottom** sits at the parent's bottom, i.e.
`child.y = parent.y + parent.h - child.h` (int-truncated). The port
(`npcs.rs:1129-1136`) places `child.y = parent.y + h` — the child's **top** at
the parent's bottom, one child-height lower per link. The follow snap repairs
positions on the next tick, but the first-tick chain shape (and any same-tick
consumer: contact damage, rendering) is wrong, and the X is not int-truncated.
Fix spec: `child.y = npcs[i].y + h - child_kind.size().1`, truncate both spawn
coords to int as the C# does.

### T2c — pass order (LOW-MED)

C# runs `AI_006` inside each NPC's `Update` in slot order; `NewNPC`'s
`Start = whoAmI` (`:51347`) makes every child take a **higher** slot than its
parent, so the head moves first and each follower snaps to the leader's
**same-tick** position. The port runs the whole follower pass (`:1173`) before
the head pass (`:1302`) — followers trail the head's **previous-tick** position
(a one-tick lag along the whole chain, visible as stretching at speed).
Fix spec: run heads first, then followers in ascending `seg_index` (or iterate
the chain head→tail per worm), keeping the death/orphan passes' current order.

### T2d — spacing overrides (LOW)

C# `num63` starts at `base.width` then: wyvern 87-92 → 42 (`:51881-51884`),
454-459 → 36, 513-515 → width−6, **412-414 → width+6** (`:51893-51896`),
621-623 → 24, EoW FTW → 62 (`:51901-51904`). The port uses own width for all —
the **crawltipede's +6 is live-kind divergence** (its segments sit 6 px too
close); the rest ride unported families/seeds.

### T2e — realLife pool (OK, re-verified)

`shares_carrier_life` (`data.rs:838-854`) excludes EoW 13-15 exactly as
`:51129-51132` forces `realLife = -1`; the pool pay/mirror passes
(`npcs.rs:1148-1170`) match `NPC.cs:81374-81378`; the stable
`worm_id`/`seg_index` carrier link is the documented S1-D-6 deviation. The
orphan rules match the C# lists: classic links die with the leader
(`:51425-51433` → port `:1242-1246` else-arm), the followerless classic link
dies (`:51468-51477` → port `:1276-1281` else-arm), the EoW head dies without a
follower (`:51491-51499` → port `:1288-1299`). One asymmetry: the C# also kills
a **classic head** whose follower is gone (`:51436-51477` includes 7/10/39/95/
117); the port relies on the shared pool making that unreachable — true today,
but the guard is absent if a future strike bypasses the pool. Note, not a fix.

## 3. Wyvern + EoW specials (T3)

### T3a — Wyvern 87-92: UNPORTED (HIGH coverage gap)

No `NpcKind` for 87-92 exists (the roster's `DD2WyvernT1-3` are the Etherian
558-560, a different family). Everything Wyvern-specific in AI_006 is absent:
- the fixed **14-link chain** with per-index kinds — legs 88 at m=1,8; body2 90
  at m=11; body3 91 at m=12; tail 92 at m=13 (`:51156-51190`);
- `flag3 = true` always (sky flyer, never digs, `:51548-51551`);
- the spriteDirection flips from velocity/step sign (`:51616-51626`,
  `:51911-51921`);
- the **altitude gates** (`flag8`, `:52053-52078`): the turn-boost when the aim
  reverses within 300 px (`velocity *= 1.1` below max), and the climb rule —
  head below the player's centre, **or the player below `worldSurface`**, or
  dead → keep horizontal speed under `num45/2` (×1.1 ramp, the `vx == 0`
  kick `-direction`) else `vy -= num46` while `vy > -num45`;
- spacing 42 (T2d), tunables 11/0.25 (T1h).
Fix spec: a kind wave (6 kinds + SetDefaults rows) then the AI_006 wyvern arms;
the chain is a fixed-count spawn like the crawltipede's, not the budget model.

### T3b — EoW split: kind swap without Transform (MED)

C# `:51511-51518`: orphaned body → `Transform(13, ai[0])`; followerless body →
`Transform(15, 0f, ai[1])`. `Transform` (`NPC.cs:81084-81145`) does more than a
type swap: reposition Y by the height delta (`:81092-81100`), `SetDefaults`
(new defense/damage/size), `TargetClosest`, **life rescale**
`life = oldLife * newLifeMax / oldLifeMax` (min 1, `:81106-81110`), and the ai[]
reset (the new head keeps the old `ai[0]` follower link; the new tail keeps
`ai[1]`; `ai[2] = 0` so a split head never re-spawns a chain).
The port (`npcs.rs:1242-1243`, `:1277-1278`) swaps `kind` only: hp stays the
body's absolute value (no ratio rescale — the port's EoW head/body/tail lifeMax
rows differ, `data.rs:2958` region), size changes without the Y reposition, no
defense/damage refresh is needed (kind-keyed tables) but the **life rescale and
the bottom-anchored reposition are behaviour**. The no-respawn property holds
by accident (seg_index > 0 blocks the budget re-roll, `:1090`).
Fix spec: on split, rescale `hp = hp * new_max / old_max` (min 1, integer
division order as C#), adjust `y += old_h - new_h`, keep the chain links.

### T3c — shadow-orb summon gate (MED)

C# `WorldGen.cs:54294-54321`: third orb (`shadowOrbCount >= 3`, or the
Don't-starve-FTW edge) → reset count, then summon **iff** `!(AnyNPCs(266) &&
crimson) && (!AnyNPCs(13) || crimson)` — a *living-boss* check, at the nearest
player to the orb. The port (`world_sim/npcs.rs:3755-3776`) polls
`shadow_orb_count >= 3 && !app.world.downed_eow` — **once the EoW is killed,
orbs never summon it again** (the C# re-summons freely while none lives); the
crimson-Heart → Brain arm is absent (documented in the comment); the spawn is
at the player, not orb-nearest-player (single-player: equivalent). The counter
itself is faithful (`tr-core/src/kill.rs:971`, save round-trip `save.rs:494`).
Fix spec: replace `!downed_eow` with `!npcs.any(EaterOfWorldsHead alive)`; the
Brain arm rides the crimson flag when the heart detection lands.

### T3d — EoW expert + fade content (MED)

- `GetEaterOfWorldsSegmentsCount` (`NPC.cs:50927-50934`): 65 classic / **70
  expert**. The port hardcodes 65 in three places (`npcs.rs:1104`, the summon
  presets `world_sim/editing.rs:7364`, `npcs.rs:3773`). Expert-mode framework
  absent — record as the expert wave's debt, but the constant must come from
  one helper when it lands.
- Expert head speed 12/0.15 (`:51696-51700`) — same debt.
- Expert **VileSpit** spawns (`:51074-51109`): body 14 above surface/no-wall
  `Next(900)` (FTW 450), head 13 `Next(90 + life/lifeMax*300)`, both
  `TargetClosest` + `CanHitLine` then `NewNPC(666)` at centre+velocity. Kind
  666 unported; expert debt.
- **Classic** alpha fade-in (`:52251-52271`): EoW segments start alpha 255 and
  fade −42/tick once moved >2 px, with the 2×dust-14 emission while fading;
  `dontTakeDamage` is NOT set for EoW (unlike 454-459). The port has no alpha
  state for the EoW — it is fully visible/solid from spawn. This one is
  classic-mode behaviour and belongs in the fix wave, not the expert debt.

### T3e — Leech tail 119 (MED)

C# chain 117 → 118 (self, budget) → **119 tail** (`:51357` type+1; 119 in the
leader-death list `:51411`). The port has no `LeechTail` kind and
`worm_next(LeechBody) = LeechBody` (`data.rs:909`) — with the budget exhausted
the same-kind arm ends the chain (`npcs.rs:1120-1124`), so **the leech grows no
tail**. Fix spec: add kind 119 (SetDefaults row), `worm_next(LeechBody) →
LeechTail`, include it in `is_worm`/`shares_carrier_life`.

### T3f — StardustWormHead 402 / kind 144 (MED)

The kind exists (`data.rs:322`) and `worm_tunables` carries its 9/0.3 row
(`:926`), but `is_worm` (`:809-828`) **excludes it** — it never enters the worm
pass, and no other dispatch claims it (census: SIM-PASS "+144" is aspirational).
Its C# arm is special: `flag3 = true` (`:51564-51567`), the `ai[2]` 600-tick
mark cycle with the velocity remap and the 50-dust burst + `SpawnStardustMark`
(`:51030-51073`), and it is a minion (no chain). Fix spec: either wire it into
the pass with a no-chain flag + its own arm, or dispatch it standalone; today
the tunables row is dead code.

### T3g — Crawltipede machine (MED)

The port's crawltipede (141-143) rides the generic head: tile-scan dig mode and
the generic steer. C# 412-414: `flag3 = true` always (`:51560-51563`), spacing
width+6 (T2d), and a wholly own aim machine (`:51777-51853`): the ground scan
under the player (±2 columns, +15 rows, `SolidTile2`), the 800-px-above-ground
aim clamp with the ±600 horizontal lead inside 500 px, the 1.3×/0.7× speed
band normalize, the no-ground 14/0.5 fallback, and the **same-type separation**
(60/400 px air, 400/1000 px grounded, push the aim away from every other
crawltipede). None of it is ported. The 30-link fixed chain IS correct in the
port's budget encoding (29 → 29 bodies + tail = the C#'s 30 links, verified
against `:51375-51391`).

## 4. Fix-wave proposal (ordering)

1. **W-A2-1 (RNG + placement, small):** T1c budget draws, T1g scan + crumble
   draw, T2b spawn placement, T2c pass order. These are stream/shape fixes with
   no new kinds; witness tests = draw-count pins per family + a two-tick chain
   geometry pin.
2. **W-A2-2 (head machine, medium):** T1a rotation, T1b three-arm steer, T1d
   surface gate, T1e corruption escape, T1f 1000-px override, T1h tunables
   (incl. leech rage). One faithful `worm_head_step` rewrite against
   `:51923-52201`.
3. **W-A2-3 (EoW + leech, medium):** T3b Transform semantics, T3c orb gate,
   T3d alpha fade (classic), T3e LeechTail kind. Expert debt (70/12-0.15/666)
   recorded for the expert wave.
4. **W-A2-4 (kind waves, large):** T3a Wyvern 87-92 (6 kinds + chain + gates),
   T3f StardustWorm wiring, T3g crawltipede machine; Digger 95-97 and
   BoneSerpent 39-41 kinds ride the same wave shape (their AI_006 arms are the
   generic ones + the T1d gate + their tunables/budget rows).

## 5. Destroyer boundary notes — for R55 (TR-wire14), NOT this lane

Shared mechanics audited above apply (pool, follow snap, orphan death). The
Destroyer-specific deltas found while reading the shared pass:

- **Link count/kinds:** C# `AI_037` spawns `j = 0..=80` — **81 links: 80 bodies
  (135) + tail (136)** (`NPC.cs:49839-49854`, `GetDestroyerSegmentsCount`
  `:50945-50952`, FTW 100). The port's same-kind chain (`npcs.rs:1103`, total
  80) yields **80 links, all `TheDestroyer`, no tail kind** — off by one and
  missing 135/136 (their SetDefaults rows exist in C# `:10680-10719`: 38×38,
  dmg 55/40, def 30/35, pool 80000, alpha 255).
- **Tail laser leak:** the port's laser arm (`npcs.rs:1194-1230`) fires on
  every same-kind link including the last; C# only type 135 fires (`:49856`),
  the tail 136 never does (the port comment admits it).
- **Laser damage:** C# `GetAttackDamage_ForProjectiles(22f, 18f)` (`:49873`) —
  expert 18; the port hardcodes 22. The RNG shape (Next(4) + Next(1400,26000)
  every tick, four Next(-20,21) inside CanHit) matches; the CanHit centres
  stand-in is documented (`:1205-1208`).
- **Probe 139:** spawned on the Mech Queen's death — two probes,
  `NewNPC(..., 139, 1, 0, 0, num, ±1)` at the Queen's centre
  (`NPC.cs:19804-19805`); SetDefaults `:10720-10729` (30×30, aiStyle **5**,
  dmg 50, def 20, life 200). **No `NpcKind::Probe` exists** (the roster's
  `MartianProbe` is 399, a different NPC). R55's wiring must add the kind +
  the aiStyle-5 servant arm + the ai[2]/ai[3] side links.
- The faithful `npc_ai_style37.rs` module is already on main UNWIRED (ROADMAP
  `S1-D-3 (aiStyle-37)` row) — R55 owns the dispatch claim; the shared-pass
  fixes W-A2-1/2 must keep the Destroyer's current routing byte-identical until
  that wiring lands (the same-kind chain quirks above are R55's to resolve).

## 6. Verification performed

- `git status --porcelain` empty in the primary before `git worktree add`;
  worktree `TR-audit5` at `c7fe175d` (= origin/main), junctions absolute.
- Whole-function reads: `AI_006_Worms` `NPC.cs:50963-52273` (every line),
  `AI_037` spawn/laser arms `:49790-49899`, `Transform` `:81084-81145`,
  `NewNPC` `:80750-80795`, `GetEaterOfWorldsSegmentsCount`/
  `GetDestroyerSegmentsCount` `:50927-50952`, probe SetDefaults
  `:10720-10729` + spawn `:19804-19805`, `WorldGen.KillTile` case 31
  `:54237-54321`; the port's `update_worms_in` + `worm_head_step` whole,
  `npc/data.rs` worm helpers whole, `carrier_index`, the orb-summon poll,
  the Worm Food summon, `break_tile`'s orb path (`kill.rs:971`), the renderer's
  rotation gate, `UnifiedRandom::next_range`.
- **No compiler invoked** (code-only mode): no cargo/rustc/rustfmt/tests/
  dotnet/graphify runs; all claims are file-read evidence at `c7fe175d`.
