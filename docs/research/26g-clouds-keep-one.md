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

## §4 — merge instructions (dispatch-arm ONE, row-dedupe)

At the d10×d19 merge (the gate's job — this branch never merges):

1. **Dispatch:** keep d19's `92 => cloud_family_step(...) + kill_step`
   arm (`projectile.rs:17384`); DELETE d10's `92 => ai_clouds(...)`
   arm. The census pin (exactly one 24-space `92 =>` arm) must hold.
   d19's `kill_step` with `tileCollide=false` is the same free
   integrate as d10's `pos += vel`.
2. **Module:** delete d10's `ai_clouds` + `set_opacity` +
   `world_point_solid` from `projectile_ai_91_100.rs` (its styles
   91/93-100 stay); the folded d19 module carries every d10 hunk
   (Item16 cue, the numeric witness pins) except the WRONG 0.25-scale
   assertion (§1.4 — do not carry).
3. **Rows:** five value-identical row sets — keep ONE (d10's consts or
   d19's literals; if d10's other arms reference `TOXIC_CLOUD` etc.,
   keep the consts and repoint d19's literals). The 511-group identity
   overlay: one copy. `proj_frames`: union (1128=5 both; 518=4/535=12
   d10-only, keep). The melee/magic overlays: one copy each.
4. **Damage walk:** the `:12448` cloud refusal + the `:13218-13222`
   toxic counter exist on BOTH branches at slightly different walk
   positions — keep ONE of each. C# order: the refusal is inside
   `Damage_CanDealDamage` (pre-walk), the counter sits between the
   113-stick arm and the 659 arm (`:13210-13223`) — d19's placement
   (post-hellfire-clamp, pre-strike, above the 99 arm) is the closer
   match; d10's counter is post-DamageVar (later than C#). Keep d19's.
5. **Witnesses:** d19's module now covers every d10 assertion worth
   keeping (rows superset, age/fade numeric pins, gas cue + cadence +
   one-shot, frame roll with the CORRECTED scale, toxic separation +
   tie-break, spore √2 tie-break, caps-all-kinds, wall mirror + brake,
   drain). Delete d10's five s92 tests with its arm; keep its s91/s93-
   s100 tests untouched.
6. **anchors.txt:** d10's `ai_clouds` canary row repoints to
   `projectile_ai_clouds.rs|pub(crate) fn cloud_family_step` (d19's
   existing row) — never delete a row to silence the canary.
7. **collision.rs:** d19's shared probe survives (§2); d10's private
   twin dies with its arm. The new probe witness stays.

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
