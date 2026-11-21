# 93 — The 949 Torch God duplicate: W4-f33 vs W4-d19 keep-one recommendation

**Status:** recommendation (code-only wave; no cargo/tests invoked).
**Branches compared:** `task/W4-f33-torch-god-favor` @ `333479e0` (the
`projectile_ai_torch_god.rs` arm, commits `ce076c9f` + `333479e0`) vs
`origin/task/W4-d19-proj-ai-181-190` @ `a4185f89` (the
`projectile_ai_sword_styles.rs` aiStyle-184 slice). **d19b is active** —
this note is the coordination hand-off: the merge keeper reads it before
deduping; neither branch deletes the other's work unilaterally.

Both branches port `AI_184_BadTorchLuck` (`Projectile.cs:42787-42822`) for
the 949 Torch God shot and add the same `proj_data` row
(`Projectile.cs:8948-8955`). The row tuples are **identical**:
`(4.0, 4.0, 184, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)`
— keep either row text, drop the other (f33's carries the
friendly/hostile-lane citation; d19's carries the ignoreWater-exclusion
citation — merge both comment halves onto the survivor).

## Arm-by-arm comparison

| C# element | W4-f33 (`projectile_ai_torch_god.rs`) | W4-d19 (`projectile_ai_sword_styles.rs`) | verdict |
|---|---|---|---|
| Owner + unlock gate (`:42789`) | `p.owner == local_owner` then `!pl.unlocked_biome_torches` inside the `Some(pl)` borrow | reads `unlocked` via `as_deref()` first, then the combined gate | equivalent |
| `SelfHurtPlayers` liveness (`:15028`) | `dead \|\| immune` (the port's `dead` flag is the C# `dead`; `active` folds — the port's player exists) | `dead \|\| hp <= 0 \|\| immune` (the extra `hp <= 0` is the pool's liveness convention, harmless) | equivalent |
| Hitbox (`Colliding`, `:15028`) | strict int-cast XNA overlap on the **live** `player.width/height` (the mount framework's hitbox, S1-P-i41) | same overlap but the **constant** `crate::player::WIDTH` for the player width | **f33 faithful** — a mounted player's hitbox width differs |
| `direction` (`:15032`) | `if pcx < proj_cx { -1 } else { 1 }` — the tie (+1) kept | `if !(pcx < c.x) { 1 } else { -1 }` | equivalent |
| `DamageVar` (`:15039`) | `damage_var(rng, p.damage, 0.0 - player.luck())` | `damage_var(rng, p.damage, -player.luck_state.luck)` (`luck()` returns the same field) | equivalent |
| `Hurt` (`:15050`) | `hurt_with_source(dmg, pcx - hit_direction, rng, true, 949)` — reproduces the C# **explicit** `hitDirection` (incl. the tie) through the knockback-sign derivation AND carries `ByProjectile(-1, whoAmI)`'s source type (the dismount-on-hit gate reads it; 949 ∉ {877,878,879} so no dismount, but the death-reason source is faithful) | `player.hurt(num2, c.x, rng, true)` — knockback sign derived from the projectile centre (same sign incl. tie), **no projectile source** (`None`) | **f33 faithful** (source parity; observation-equal for 949 today) |
| Post-hurt `StatusPlayer` (`:15050-15053`) | returns the `Hurt > 0` bool (the gate shape kept; no 949 arm exists in `StatusPlayer`) | folds silently (documented) | equivalent |
| Item8 latch (`:42793-42797`) | `local_ai[0]` 0→1 + `wants_sound (2, 8, center)` | identical | equal |
| Budget drain (`:42798-42805`) | XNA `Length` form: f32 sum → f64 sqrt → f32 cast (the `npc_ai_style64` precedent); if/else kept — the collide arms the tick **after** the budget goes ≤ 0 | `p.vel.length()` (glam's f32 sqrt — a precision deviation from XNA's double intermediate); same if/else | **f33 faithful** (float form) |
| 4-dust trail (`:42806-42821`) | **full census**: per dust the six `Dust.NewDust` draws (`Dust.cs:170-175`/`:190`) + the torch-family seventh `Next(-10,6)` (`:214-219`, every `TorchID.Dust` id except 156/234/66) + the explicit `Next(3)` = **32 draws/tick** (28 for 156/234/66) — the `npc_ai_empress`/`projectile_ai_empress` exact-draw precedent | **explicit-only census**: the four `Next(3)` gates, the `NewDust` internals skipped with the dust layer (d19's house convention) = **4 draws/tick** | **f33 faithful to the projectile-AI census precedent** — d19's skip loses 28 draws/tick/shot on the shared `UpdateProjectiles` stream (every later draw in the pass shifts) |
| `Kill()` census (`:70976-70995`) | ported: Item10 cue + 20 dusts × (6 + family 7th + `Next(3)`) = **160 draws** (140 rainbow), wired at the pool's death hook with the out-of-world-guard exclusion (`:16696` deaths draw nothing) | **absent** — no 949 Kill arm, no death-hook wiring | **f33 only** |
| PVE/EVP lane exclusions | explicit: the hostile-contact walk exempts 949 (the hurt rides the AI's own `SelfHurtPlayers`), the PVE walk refuses it (`friendly = false`, `:12611`) | none added (relies on the row's stock flags reaching the lane gates) | **f33 safer** — the port's lanes are gate-shaped, not flag-shaped |
| Dispatch movement | `kill_step` only while `active` (the 184 AI never kills, so equivalent) | `kill_step` unconditional (their doc cites `HandleMovement` before the `!active` return, `:17015`/`:17027`) | equivalent for 184 |
| Witness correctness | `333479e0` fixed the budget witness: the else arm fires the tick **after** `ai[1]` goes negative | `w4_d19_the_torch_fireball_drains_and_rearms_collision` asserts `p.tile_collide` **on the drain tick itself** (`ai[1] = 3`, speed 5 → −2, then `assert!(p.tile_collide)`) — the arm's if/else would NOT arm it that tick; **the witness is wrong and would fail if run** | **f33 correct** |
| Doc citation hygiene | `SelfHurtPlayers` cited `:15025-15062` throughout | the module header mis-cites `SelfHurtPlayers` as `:41810-41846` (that window is `FindChlorophyteSporeCloudSpawn`); the fn-level doc has the right lines | f33 clean |

## Recommendation: KEEP W4-f33, DROP d19's 184 slice

Keep `projectile_ai_torch_god.rs` + its dispatch arm + the Kill-census
wiring + the lane exclusions (the f33 branch). From d19's
`a4185f89` drop ONLY:

1. the `// aiStyle 184` section of `projectile_ai_sword_styles.rs`
   (`torch_god_self_hurt` + `ai_bad_torch_luck`, ~lines 389-478) and its
   witness `w4_d19_the_torch_fireball_drains_and_rearms_collision`;
2. the `184 =>` dispatch arm in `projectile.rs` (f33's arm replaces it —
   both call after the same `187` arm; keep ONE);
3. the duplicate `TORCH_GOD` const + row in `projectile/data.rs` (keep
   one const, one row; merge the comment halves).

Everything else in d19 (182/183/185/186/190 + their 12 other rows) is
untouched by this recommendation. The d19 module's header census line for
184 should point at `projectile_ai_torch_god.rs` after the dedupe.

**Why not keep d19's:** the 28-draws/tick + 160-draws/death census loss
breaks the exact-draw discipline every other projectile-AI module
follows (`npc_ai_empress`, `projectile_ai_empress`, `prefix.rs`'s NewDust
census); the missing Kill arm loses the Item10 cue + the death burst
entirely; the width-constant hitbox and the glam `length()` are small but
real fidelity gaps; and the shipped witness contradicts the C# if/else.

**Merge mechanics:** whichever branch lands second rebases the dedupe;
the anchors.txt rows of both branches name their own module — the d19
row (if it names the 184 arm) repoints to the f33 survivor, never deleted
outright (the canary rule).

## Census pins (both branches agree)

- Row: 4×4, aiStyle 184, `penetrate = -1`, `tileCollide = false`, stock
  `timeLeft 3600` / `extraUpdates 0`, `friendly`/`hostile` false
  (`Projectile.cs:8948-8955` + the stock defaults `:764`/`:771`/`:787`/`:788`).
- `TorchID.Dust` (`TorchID.cs:179-184`): the 24-id table is verbatim in
  f33 (`TORCH_DUST`); d19 reads the dust id from `ai[0]` without the table
  (the spawner passes it — f33's `torch_god.rs` launch carries
  `TorchID.Dust[clamp(frameY/22, 0, 23)]`, `Player.cs:17888-17889`).
- The 949 never dies on a player hit (`penetrate = -1`, no kill in
  `SelfHurtPlayers`); the hurt is owner-gated and stops once
  `unlockedBiomeTorches` rises (`:42789`).