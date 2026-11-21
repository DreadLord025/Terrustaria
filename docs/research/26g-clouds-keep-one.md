# 26g — aiStyle-92 cloud family KEEP-ONE (W4-d19 × W4-d10 dup adjudication)

**Round.** W4-d19 follow-up 2 (code-only, branch
`task/W4-d19-proj-ai-181-190`, base `1054c396a`). The d10 branch
(`task/W4-d10-proj-ai-91-100`, tip `1f0b60e96`, arm landed `b34ad73c5`)
ported the SAME inline `AI()` arm `Projectile.cs:32558-32820` into
`projectile_ai_91_100.rs::ai_clouds` that this branch's
`projectile_ai_clouds.rs::cloud_family_step` ports. Both were read WHOLE
against the C# arm and every cited helper; this note is the arm-by-arm
adjudication + the merge instructions.

## Verdict

**KEEP `projectile_ai_clouds.rs` (d19) as the canonical aiStyle-92
machine.** It is the faithful superset on the load-bearing axes:

| axis | d10 `ai_clouds` (tip 1f0b60e9) | d19 `cloud_family_step` | winner |
|---|---|---|---|
| `IsWorldPointSolid` probe | module-private `world_point_solid` (d10 :100-145) | shared `collision::is_world_point_solid` (collision.rs:819) | **d19** (see §2) |
| gas player buff | gated `slots[i].owner == local_owner` (d10 :406) | ungated `Option<&mut Player>` (C# `Main.LocalPlayer`, `:32800-32807` — NO owner gate) | **d19** — d10's gate is invented; a non-local-owned gas (net future / trap wiring) would wrongly skip the local player's Poisoned |
| GasTrap Item16 cue | `wants_sound = Some((2,16,cx,cy,0))` at the pre-growth centre (d10 :196-197) | was missing → **FOLDED into d19 this round** | d10 hunk folded |
| 190 spore-child drain | none (d10 has no 1126 lane) | `drain_claymore_spores` (1128 centred + ai[1] seed; 1129 data-seam contract) | **d19** |
| damage-walk slices | `:12448` refusal + `:13218` counter, placed after `target_allowed` / post-DamageVar | same two slices, refusal after the 443 gate, counter post-hellfire-clamp pre-strike | equivalent — ONE copy at merge (§4) |
| witnesses | 5 (rows / age-fade / gas / frame-roll / toxic-separation) | 7 + this round's folds (rows / frame-roll+bloom / lifetime-caps-all-kinds / spore-repulsion+√2-tiebreak / wall-mirror+toxic-brake / gas-cadence+cue+one-shot / drain / toxic-fade-spin / toxic-separation) | d19 superset after fold |

### Arm-by-arm vs the C# (`Projectile.cs:32558-32820`)

Both copies were checked line-for-line; where they differ:

1. **`:32563-32566` frame roll** — identical in both: one
   `Next(projFrames[1128]=5)` while `ai[0]==0` (the ONLY draw;
   `Main.cs:8726` re-verified).
2. **`:32568-32579` gas one-shot** — d10 correct shape (outer gate
   `localAI[0]==0`, INNER `direction==0` fix, broadcast every first
   tick); d19's old `&& direction == 0` outer gate was a latent defect
   (stock spawn direction 1 would suppress the block) — **fixed this
   round** to the d10/C# shape, plus the Item16 cue fold. The
   `Spawn_GasTrap` body (`ParticleOrchestrator.cs:2911-2953`, 12+10
   particles) is the particle-layer skip in both; its draws go with the
   unported pool (dust-body convention). The cue plays at
   `PositionInWorld = base.Center` captured BEFORE the `:32605` resize.
   Note `localAI[0]` is never set to 1 — the one-shot rides the
   `:32624` spin accrual (`direction` ±1 ≠ 0 from tick 2).
3. **`:32580-32599` triple-age + caps + post-kill pin** — identical
   (260/80/ai[1]-or-220; `ai[0]` pinned AT the cap after `Kill()`).
   Both keep the kill-tick integrate (`:17015` HandleMovement before
   the `!active` return `:17027`; `tileCollide=false` → free step).
4. **`:32600-32623` scale/opacity ramps** — identical maths. d10's
   gas height recomputes `(int)(50*scale)` vs d19 `h = w`: same value
   (C# `:32606` recomputes too — equal by construction). The Opacity
   setter (`:546-549` clamp-truncate) identical in both.
   **d10 witness defect found:** `w4_d10_s92_the_spore_cloud_rolls_its_frame_once`
   asserts `scale ≈ 0.25` / `w == 6` at tick 1, calling it "the ramp's
   clamped start". WRONG: `Remap(1/220, 0, 0.1, 0.25, 1.1)` →
   `GetLerpValue(0, 0.1, 0.004545) = 0.04545` (INSIDE the from-range, no
   clamp) → scale ≈ 0.28864, `w = (int)(30·0.28864/1.1) = 7`. d19's
   formula-based witness is correct; the folded d19 pin now asserts the
   re-derived 0.28864 explicitly. Do NOT carry d10's assertion over.
5. **`:32624-32625` spin** — identical (`whoAmI·0.4002029 +
   localAI[0]·2π/480`). d19's index-seed witness (slot 2) folded from
   d10's numeric pins (alpha 252 at tick 1, rot seed).
6. **`:32626-32637` drags** — identical (zero / ×0.97 / ×0.96).
7. **`:32638-32670` toxic repulsion** — same maths (int-box intersect,
   index tie-break, `SafeNormalize(UnitX)·0.005`, 0.6-Lerp BOTH slots
   live). d10 latches the self-centre pre-scan; the C# re-reads
   `base.Center` per iteration (`:32652`) — d19's per-pair re-read is
   verbatim; the difference is unreachable (the scan writes only
   velocities, never positions). d10's pair witness folded into d19.
   **Second d10 witness defect found:** the tie-break test asserts
   `slots[0].vel.x > 0 && vel.y < 0` — backwards. The j>i delta is
   (+1,−1) but SELF lerps toward `velocity − push` (`:32667`), so self
   moves (−1,+1) and the SIBLING takes (+1,−1). The folded d19 witness
   asserts the re-derived signs on both slots.
8. **`:32671-32705` spore repulsion** — same maths (width-falloff
   push, plain `Normalize` after the tie-break — the √2-length
   tie-break factor kept; d19 witnesses it, d10 does not). d10 latches
   `width` pre-scan; C# re-reads `base.width` (`:32701`) — again
   unreachable (no width writes in the scan). d19 verbatim.
9. **`:32706-32783` wall probe + reflection ladder** — same ladder
   (X-mirror + −16/+16/−32/+32 probes, Y-mirror + −16/+16, 0.5-Lerp
   fallback). d19 uses safe-normalize on the probe deltas (documented
   zero-delta fold, yoyo precedent); d10 uses raw `normalize()`
   (NaN-on-zero, the C# literal). The zero delta needs centre == probe
   after a solid hit — geometrically unreachable in-world (the probe
   is 16 px off-centre); d19's fold stays, documented.
   **Probe internals differ — §2.**
10. **`:32784-32819` gas tail** — same 20/2940/180-px/15-tick machine;
    d10's owner gate is invented (verdict table); d10's
    `!(localAI[1] <= 0)` → `local_ai[1] > 0` rewrite differs only on
    NaN (unreachable). `ApplyBuffTo`'s deadMansSweater halving
    (`:12393-12396`) folds out in both (no port field).
11. **Rows (`data.rs`)** — d10's five rows and d19's five rows are
    value-identical (511/512/513: 32×32/40×38/30×28, idStatic-10,
    identity 511, alpha 255, scale 1.1, magic; 1007: 50×50, trap,
    alpha 0; 1128: 30×30, tileCollide false, melee, idStatic-30).
    Both add the 511-group identity overlay and `proj_frames[1128]=5`
    (d10 also adds 518=4/535=12 for ITS styles 94/100 — keep those).
    d10 uses named consts (`TOXIC_CLOUD`…`SPORE_CLOUD`), d19 literals —
    at merge keep ONE row set; prefer d10's consts (its module's other
    arms may reference them) but the VALUES are already equal.

## §2 — `IsWorldPointSolid` (`Collision.cs:1434-1491`) dup-check

d10 HAS a copy: module-private `world_point_solid` in
`projectile_ai_91_100.rs` (tip lines 100-145). d19's copy is the shared
`collision.rs::is_world_point_solid` (:819). **Keep d19's; delete d10's
at merge.** Re-derived differences:

- **Tile coordinates:** C# `ToTileCoordinates` = `(int)X >> 4`
  (`Utils.cs:1807-1810`) — floors negatives. d10 does `(pos.x / 16.0)
  as i32` — truncates toward zero. Unreachable behind the fluff-1
  `InWorld` gate (both refuse), but d19 is structurally verbatim.
- **Solidity:** d10 reads the raw `TILE_SOLID` set — counts leaf-block
  192 solid. The LIVE `Main.tileSolid[192]` is cleared after worldgen
  (`WorldGen.cs:15715`); d19's `solid_kind` carries the exception.
  **d19 correct, d10 wrong for 192.**
- **Platform gate:** C# also has `type <= TileID.Count`; both ports
  drop it (the sets are bounded anyway) — equal.
- **`:1470` always-false slope guard:** d10 keeps the literal dead
  `if`; d19 folds it out with the citation. Behaviour identical; d19's
  doc now records the adjudication.
- d19 gained a full witness this round
  (`collision.rs::the_world_point_solid_probe_is_verbatim`): closed
  edges, half-brick, slope-1 triangle, actuated, platform-19 +
  planter-380 both modes, leaf-192, fluff-1 bounds, negative floor.
  d10 has NO probe witness.

No OTHER `IsWorldPointSolid` copy exists on either branch (grep
`is_world_point_solid|world_point_solid` — d10's one private fn, d19's
one shared fn).

## §3 — the f33 spawn-call contract (UPDATED — f33 is NOW on origin)

The previous round's probe pre-dated the push: `task/W4-f33-torch-god-favor`
is on origin at **`8a000407f`** (event machine `ce076c9f9` + witness fix
`333479e0b` + biome-torch arms `a429b37c0`/`8a000407f`). Read whole:
`torch_god.rs::torch_attack_step`/`drain_pending` +
`projectile_ai_torch_god.rs` + the 949 row.

**Compose verdict: the spawn call matches the d19 184-arm's contract
EXACTLY.**

- C# `TorchAttack` (`Player.cs:17886-17903`): spawn point
  `(torchX*16+8, torchY*16)`, `num9 = (Center−spawn).Length()` BEFORE
  normalize, velocity = normalized × 8, damage
  `(int)(40 × HostileProjectileDamageMultiplier)`, ai0 =
  `TorchID.Dust[clamp(frameY/22, 0, Count−1)]`, ai1 = num9, knockback
  1f, owner = whoAmI; `friendly` stays stock false; `netUpdate = true`.
- f33's `drain_pending` calls
  `spawn_with_data_ai_owned(949, local_owner, launch.x, launch.y,
  launch.vx, launch.vy, launch.damage, 1.0, dust_id as f32, distance)`
  then sets `friendly = false` + `net_update = true` — every argument
  in the C# order; `launch_aim` keeps the pre-normalize length (f64
  sqrt cast back, the XNA `Length` form) and the reciprocal-multiply
  normalize (NaN on zero, no guard — the C# has none).
- d19's arm (`projectile_ai_sword_styles.rs::ai_bad_torch_luck`) reads
  exactly those slots: ai0 = the dust TYPE (`(int)ai[0]`, `:42806`),
  ai1 = the flight-distance budget (`:42798-42805` drain then arm
  tileCollide), the owner-gated self-hurt with `dodgeable: true`
  (belt witness `w4_d19_the_torch_fireball_self_hurt_is_dodgeable`).
  The 949 rows on BOTH branches are value-identical
  (`4×4, 184, −1, 3600, 0, false×3, RESET_STATE`, `:8948-8955`).
  **No projectile-side change needed — the contract holds.**

**BUT the two branches carry DUPLICATE 949 machines that DIVERGE —
a second keep-one, adjudicated for the merger (out of this round's
edit scope: f33 owns the event lane):**

| slice | d19 `ai_bad_torch_luck` | f33 `ai_184_bad_torch_luck` | keep |
|---|---|---|---|
| self-hurt | `player.hurt(dmg, c.x, rng, true)` — knockback from the fireball centre; NO projectile source | `hurt_with_source(dmg, pcx − hit_direction, rng, true, 949)` — reproduces the C# explicit `hitDirection` incl. the equality tie (`!(pcx < proj_cx)` → +1) + the dismount-on-hit source gate (`Player.cs:38396-38400`) | **f33** |
| luck read | `player.luck_state.luck` | `player.luck()` (same value, the accessor) | either |
| speed | glam `vel.length()` | XNA form `(x²+y² as f64).sqrt() as f32` | **f33** (verbatim) |
| dust trail | 4× `Next(3)` only (dust-body skip convention) | 4× (NewDust 6-draw census + family 7th + `Next(3)`) — the FULL `UpdateProjectiles` stream consumption (`Dust.cs:170-219`) | **f33** — d19's skip UNDER-DRAWS the shared stream (28/32 draws per tick missing); once f33 merges, its census is the faithful one |
| Kill census | none | Item10 + 20-dust census (`:70976-70995`) wired at the death hook with the out-of-world guard | **f33** |
| EVP/PVE lanes | none (the row's friendly=false default + the pool's refusals) | explicit 949 exemptions (`Damage_EVP` `:14775`, `Damage_PVE_Inner` `:12611`) + the spawn-side `friendly=false` pin | **f33** |

Merge instruction: keep f33's `projectile_ai_torch_god.rs` as the 949
machine; retire d19's `ai_bad_torch_luck` + `torch_god_self_hurt` and
repoint the 184 dispatch arm; KEEP d19's dodge witness (repointed at
f33's fn — the belt assertion holds for both) and d19's 949 row (equal
values; one copy). The contract-note in ANALYSIS is updated to record
the verified compose + this adjudication.

## §4 — FINAL drop-hunks (published for merger/d10b; keep-ours = d19)

**Re-pinned to the d10 LATEST tip `732d3480b`** (the doc-sweep `732d3480b`
+ crosschecks `dc7acb5e8` + yoyo-string grants `c6fd3bc03` + mouse-delta
un-park `29a73a1d6` landed AFTER the `1f0b60e96` probe and shifted every
line below; the `1f0b60e96` pins in the earlier round are SUPERSEDED).
Every range was re-read whole at `732d3480b`. **Keep-ours verdict
unchanged: d19's `projectile_ai_clouds.rs` is the canonical aiStyle-92
machine (§1); the shared `collision::is_world_point_solid` survives
(§2).** The consts question is now RESOLVED by census: a full-tree grep
of `TOXIC_CLOUD|GAS_TRAP|SPORE_CLOUD` at `732d3480b` shows EVERY consumer
sits inside the drop/edit hunks below — no surviving d10 arm references
the consts, so **keep d19's literals wholesale and retire d10's consts**
(supersedes the earlier "prefer d10's consts" hedge).

At the d10×d19 merge (the gate's job — this branch never merges):

### 4.1 `crates/tr-app/src/projectile_ai_91_100.rs` @ `732d3480b`

| drop | lines | content |
|---|---|---|
| D1 | **109–442** | the contiguous cloud block: `world_point_solid` doc+fn (109–163), `set_opacity` doc+fn (165–169), `ai_clouds` doc+`#[allow]`+fn (171–440), the trailing double blank (441–442). Line 443 (`ai_shadowflame`'s doc) is the keep boundary. |
| D2 | **106** (edit) | remove `remap` from the `use crate::projectile::{…}` list — its ONLY module call-sites (242/249/250/255/260/261/266/267/412) die with D1. `proj_frames` STAYS (the 94 portal reads it, :712); `can_hit/chaseable/npc_center/safe_normalize` STAY (99/100); `Npc` STAYS (93). The `#[allow(unused_imports)]` (:104) makes this cosmetic, not load-bearing. |
| D3 | **1849–1981** | the five s92 witnesses with their docs: `w4_d10_s92_the_five_rows_are_verbatim` (1849–1878), `…the_cloud_ages_and_fades` (1880–1904), `…the_gas_trap_grows_and_poisons` (1906–1933), `…the_spore_cloud_rolls_its_frame_once` (1935–1958, the WRONG 0.25-scale assertion — §1.4, do NOT carry), `…the_clouds_separate` (1960–1980, the WRONG tie-break sign — §1.7), blank 1981. Line 1982 (the 514-row doc) is the keep boundary. The s91/s93–s100 tests stay untouched. |

The folded d19 module already carries every d10 hunk worth keeping
(Item16 cue, the numeric witness pins with the two re-derived
corrections). The module-doc census row `| 92 | … |` (:12) and the RNG
contract's `- 92:` bullet (:42–46) should gain a one-line "the 92 arm
retired onto d19's `projectile_ai_clouds` (research/26g §4)" note —
documentation, not behaviour.

### 4.2 `crates/tr-app/src/projectile.rs` @ `732d3480b`

| drop | lines | content |
|---|---|---|
| D4 | **18198–18219** | the `92 =>` dispatch arm with its W4-d10 comment (18198–18206 comment, 18207–18219 arm incl. the `pos += vel` free integrate). Keep d19's arm (`projectile.rs:17385–17398` at d19 tip `a3481f7c9`): `cloud_family_step` + `kill_step` — the same free integrate, and the census pin (exactly one 24-space `92 =>` arm) holds. |
| D5 | **23027–23039** (edit, NOT whole-drop) | the combined `:12448` refusal. The cloud terms (23033–23035: `TOXIC_CLOUD…SPORE_CLOUD && ai[1] >= 1` + `GAS_TRAP`) DROP — d19's refusal survives (`projectile.rs:22790–22800`, literals). The **`MEDUSA_HEAD` term (23036) MUST SURVIVE** (d10's style-100 marker, no d19 twin): reshape to `if p.kind == MEDUSA_HEAD { continue; }` and trim the comment (23027–23032) to the Medusa sentence. |
| D6 | **23230–23237** | the toxic hit-counter (comment 23230–23233 + the `TOXIC_CLOUD…` if 23234–23237). Keep d19's (`projectile.rs:22980–22989`): post-hellfire-clamp, pre-strike — the closer C# match (`:13210–13223` sits between the 113-stick and 659 arms; d10's post-DamageVar placement is later than C#). The adjacent 524 ichor arm (23238–23244) STAYS (d10 style 96). |

### 4.3 `crates/tr-app/src/projectile/data.rs` @ `732d3480b`

| drop | lines | content |
|---|---|---|
| D7 | **714–727** | the five consts with their docs: `TOXIC_CLOUD/_2/_3` (714–719), `GAS_TRAP` (720–723), `SPORE_CLOUD` (724–727). No surviving consumer (the census above). `SHADOW_FLAME` (:713) and `NAIL_FRIENDLY` (:728+) stay. |
| D8 | **1721–1752** | the five rows with their comments: toxic trio (1721–1737), gas (1738–1744), spore (1745–1752). Value-identical to d19's (`data.rs:3748–3762` at d19 tip) — keep d19's literal-keyed set. |
| D9 | **4287–4288** (edit) | the `immunity_identity` else-if branch (`TOXIC_CLOUD… → TOXIC_CLOUD`). Keep d19's (`data.rs:4243–4244`, `511|512|513 → 511`). Trim the comment's "W4-d10: the Toxic Cloud trio…" sentence (4281–4282). |
| D10 | **4465–4467** (edit) | the melee overlay: drop `SPORE_CLOUD | ` from 4467, KEEP `| ICHOR_SPLASH` (d10 style 96); reword the comment (4465–4466) to the ichor-only sentence. d19's `| 1128` (:4432) survives. |
| D11 | **4533–4537** (edit) | the magic overlay: drop the `TOXIC_CLOUD | TOXIC_CLOUD_2 | TOXIC_CLOUD_3` terms from 4536, KEEP `SHADOW_FLAME` + `MEDUSA_HEAD` (4536–4537); reword the comment (4533–4535). d19's `| 511 | 512 | 513` (:4500) survives. |
| D12 | **4789** (edit) | `1128 => 5` in `proj_frames` — d19 has it (`data.rs:4851`). KEEP `518 => 4 / 525 => 5 / 535 => 12` (4786–4788, d10-only styles); trim the spore sentence from the comment (4782–4785). |

### 4.4 `anchors.txt` @ `732d3480b`

| edit | line | content |
|---|---|---|
| D13 | **900** | the `projectile_ai_91_100.rs|pub(crate) fn ai_clouds` canary row REPOINTS to `crates/tr-app/src/projectile_ai_clouds.rs|pub(crate) fn cloud_family_step` (d19's existing row, anchors.txt:900 at d19 tip) — merge the two descriptions into ONE row; never delete a row to silence the canary. The d19 collision-probe row (anchors.txt:901) stays. |

### 4.5 unchanged adjudications

- **collision.rs:** d19's shared probe survives (§2, `collision.rs:825`
  at d19 tip); d10's private twin dies inside D1. The probe witness
  (`the_world_point_solid_probe_is_verbatim`, `collision.rs:2565`) stays.
- **Witnesses:** d19's nine cloud tests + the collision probe cover
  every d10 assertion worth keeping (D3's two defects re-derived and
  corrected in the folds — §1.4/§1.7).
- **Post-drop compile surface:** D1+D3 remove every `ai_clouds` /
  `world_point_solid` / `set_opacity` reference in the module; D4–D6
  remove every `projectile.rs` reference; D7–D12 remove every const
  consumer. No other d10 file names the cloud symbols (the full-tree
  grep). The `remap` helper stays `pub(crate)` on d19 (`projectile.rs:
  14146`) — d10's own `remap` import dies with D2, the pool's copy is
  untouched.

## §5 — the 1129/d21 merge-contract (re-verified at d21 tip `9fa9e9ba0`)

d21's latest tip is `9fa9e9ba0` (docs) over the port `20fe4b3ed`. Read
at that tip:

- **Row** (`data.rs:2962`, `CHLOROPHYTE_CLAYMORE_BLADE_SLAM = 1129`):
  `(16.0, 16.0, 209, -1, 5, 0, false, false, false, (true, -1, false,
  -1, 255, 1.0))` — every field against `:10580-10596`: 16×16, aiStyle
  209, penetrate −1, timeLeft 5, tileCollide false, friendly (stock),
  alpha 255, the LOCAL pair (−1), scale 1.0 stock; `ignoreWater` rides
  the overlay (`:4298`, `:10587`), melee the overlay (`:4335`,
  `:10593`). `ownerHitCheck` (`:10594`) is the recorded spear-seam
  deviation (d21's comment). VERBATIM.
- **Arm** (`projectile_ai_201_210.rs:117-130`): `w = (int)ai[1]`,
  `h = (int)ai[2]`, `ai[0]++`, `>20 → Kill` — verbatim against
  `AI_209_ChlorophyteClaymoreBladeSlam` `:69622-69631`, zero draws.
- **Dispatch** (`projectile.rs:19891`): `209 => ai_claymore_blade_slam
  + pos += vel` — the free integrate (tileCollide false), ONE arm.
- **Compose with the d19 drain:** `drain_claymore_spores` spawns
  `spawn_with_data_ai_owned(1129, owner, x − row.w·0.5, y − row.h·0.5,
  0, 0, damage, 0.0, 0.0, 128.0)` + `set_ai2(96)` — the C# `:41967`
  call `(vector6, Zero, 1129, (int)(damage·1f), 0f, owner, 0f, 128f,
  96f)` with `NewProjectile`'s centre convention (`:10876-10877`,
  16×16 → −8). ai0 = 0 (the age starts fresh), ai1 = 128 → w = 128,
  ai2 = 96 → h = 96 on the child's first AI tick. The full damage
  rides. The both-states witness
  (`w4_d19_the_spore_drain_centres_the_clouds_and_parks_the_slam`)
  holds pre-merge (row absent → no-op) and post-merge (row present →
  the centred spawn + the ai pins). **Contract GREEN — no change.**

## §6 — the 181–190 band FINAL table (PROJ-181 phase 2)

Census at d19 tip `a3481f7c9` + the pushed cross-branch tips (d18
`18916087b`, d12c `9e668d0f0` on `task/W4-d12-proj-ai-111-120`, d20
`243f81672`, f33 `7b334ea74`, d21 `9fa9e9ba0`, main `a725acb04`).
"ARMED" = a dispatch arm serves the style; "ROWED" = every kind the arm
serves has a `proj_data` row (the spawn seam declines rowless kinds).

| style | kinds | arm (file:line at its branch) | dispatch | rows | status |
|---|---|---|---|---|---|
| 181 | 932 | `projectile_ai_misc.rs:185 ai_fairy_queen_shot` (main, S1-D-4) | `projectile.rs:17331` | 932 `data.rs:3653` (main) | **ARMED+ROWED** |
| 182 | 933/1100 | `projectile_ai_sword_styles.rs:257 ai_final_fractal` (d19 `a4185f89`) | `projectile.rs:17348` (no integrate — the `:20435` skip list) | 933/1100 `data.rs:3680` | **ARMED+ROWED** |
| 183 | 880/929 | `…sword_styles.rs:373 ai_zoologist_strike` (d19) | `projectile.rs:17371` | 880/929 `data.rs:3671` | **ARMED+ROWED** |
| 184 | 949 | `…sword_styles.rs:443 ai_bad_torch_luck` (d19) | `projectile.rs:17403` | 949 `data.rs:3687` | **ARMED+ROWED — keep-f33** (§3: f33's `ai_184_bad_torch_luck` owns the 949 machine at merge; d19's arm retires, the dodge witness repoints; the rows are value-identical). f33 IS on origin (`8a000407f` ancestor of tip `7b334ea74`; the closeout re-pins the d19 drop-hunks). |
| 185 | 476 | `…sword_styles.rs:494 ai_life_drain` (d19) | `projectile.rs:17415` | 476 `data.rs:3664` | **ARMED+ROWED** |
| 186 | 950 | `…sword_styles.rs:553 ai_princess_weapon` (d19) | `projectile.rs:17427` | 950 `data.rs:3692` | **ARMED+ROWED — overlay landed**: magic `data.rs:4497`, ignoreWater `:4385–4386`, the `:13198` owner-facing direction arm `projectile.rs:23001` (shared with 476). |
| 187 | 964/965 | `projectile.rs:10640 ai_insanity_shadow` (main `7bfd6afa4`, S1-D-4b-1) | `projectile.rs:17689` | 965 `data.rs:2158` (main); **964 rowed on d18** `18916087b` (`INSANITY_SHADOW_FRIENDLY`, d18 `data.rs:3944`) | **ARMED+ROWED** (964's row rides d18's merge; the arm already serves it) |
| 188 | 974 | `projectile.rs:10745 ai_lights_bane` (main) | `projectile.rs:17693` | 974 `data.rs:3870` (main) | **ARMED+ROWED** |
| 189 | 978/1125 | `projectile.rs:11245 ai_volcano` (main) | `projectile.rs:17697` | 1125 `data.rs:2912` (main); **978 rowed on d12c** `9e668d0f0` AND d18 `18916087b` | **ARMED+ROWED — 978 keep-one**: d12c and d18 both row 978 (value-identical: 160×160/189/pen 3/timeLeft 30 — the `:9268` second write wins — local −1/alpha 255/ignoreWater/melee). **Keep d12c's** — the superset: it alone adds `proj_frames[978]=5` (`Main.cs:8493`, re-read verbatim this round; d18's copy lacks the frames row, so its 2-tick frame machine would wrap at the stock 1). |
| 190 | 972/982/983/984/997/1043/1126 | `…sword_styles.rs:1038 ai_nights_edge` (d19) | `projectile.rs:17438` (+ the `drain_claymore_spores` child drain `:17460`) | all seven `data.rs:3700–3736` | **ARMED+ROWED** — the 1043 row is ALSO on d20 (r109's dup note): keep-one at merge (values equal). The use-path gap is T5 (§7), not an arm/row gap. |

**UNARMED_ROWED left in the band: NONE.** Every style 181–190 has both
a dispatch arm and rows for every kind it serves. The residuals are
merge-order, not coverage:

1. **964** (187) and **978** (189) rows live on UNMERGED branches (d18 /
   d12c) — until they merge, spawns of those kinds decline at the seam
   (the arms are already on main and serve them the moment the rows
   land). The 978 dup adjudicates keep-d12c (the frames superset).
2. **184** keeps the d19 arm until the f33 merge retires it (§3) — no
   coverage hole either way (both machines are faithful; f33's is the
   fuller census).
3. **190 is AI-complete but use-path-inert** until T5 lands (§7) — the
   rows + arm are correct; the missing piece is the item-use spawner,
   a different lane.

## Witnesses (this round, WRITTEN NOT RUN — code-only)

- `projectile_ai_clouds.rs`: +2 new
  (`w4_d19_the_toxic_cloud_fades_and_spins_from_its_index`,
  `w4_d19_the_toxic_clouds_separate_symmetrically` — the d10 folds),
  3 extended (gas cue + one-shot + growth pin; the corrected 0.28864
  bloom pin; the rows' gas/spore penetrate pins).
- `collision.rs`: +1 new
  (`the_world_point_solid_probe_is_verbatim`).
- `projectile.rs`: the `remap` helper goes `pub(crate)` (the d10
  visibility — the clouds module now shares it, no private lerp twin).

No cargo/rustc/rustfmt/tests/graphify invoked (code-only wave).
Verification = whole-block C# re-reads (`:32558-32820`,
`Collision.cs:1434-1491`, `Player.cs:17813-17910`, `:42787-42822`,
`:15025-15062`, `:69622-69631`, `ParticleOrchestrator.cs:2911-2953`,
`Utils.cs:1807-1810`) + hand arithmetic (the 0.28864 re-derivation,
the alpha-252 truncation, the bit-set membership checks for
1/19/192/380).
