# 26d — Projectile `Kill()` / on-hit / tile-collide hook census (W4-g1, lane a) — SEALED

> **SEALED 2026-11-21 (the d78-fold round).** The lane's final ownership:
> **g1 KEEPS 459/521/522/80** (the shared 459|709 burst, the 521 split, the
> 522 row/arm/bounce/Kill-census, the 80 tile break); the hostile EVP trio
> **452/454/448 DROPPED from g1** per the d78 FINAL adjudication
> (`26f-d78-final-kill-dedupe.md` §1, d78 tip `fd59c51a`) — d78's
> `hostile_death_blast` + `damage_evp_boxed` port `Damage_EVP` verbatim
> (g1's first-round hit was raw-damage and its General immune gate would
> refuse the boss blasts the C# `:14799` lands); g65's pushed bodies own
> the trio's draws. The 459|709 pool-gate widening EXECUTED on this branch
> (was "at the merge" — the local const matches f20's byte-for-byte). The
> family ledger: `373-g1-proj-missing-a-family-bible.md` (renumbered
> 26d-g1 → 373 per research-number-registry §10). g1 witnesses:
> **12 WRITTEN NOT RUN** (code-only; the DONE tip carried 18 despite its
> "22" message — 4 module + 3 pool hostile-trio witnesses dropped with
> their hooks, one 709-gate witness added: 6 module + 6 pool).
>
> Researched 2026-11-21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> The f19 aiStyle 1-105 range is EXHAUSTED (every missing style is claimed by a
> live d-wave branch — see `26c`), so this lane falls back to the task's second
> arm: census the C# `Kill()` / on-hit / tile-collide hooks that have NO Rust
> counterpart, and port the UNCLAIMED functional ones verbatim + witnesses.

## Method

- In-flight probe first: `git fetch --prune` + `git branch -r` + `git worktree
  list`. The branch IS the claim (AGENTS.md). Every pushed `task/W4-*` branch
  was diffed against `origin/main` for `projectile*.rs`; none touches the six
  hooks below (the death-hook block `projectile.rs:19946-20068` is untouched by
  a3/a4/a9/c6/c13/d16/d80/d91/d116 — verified by `@@` hunk scan).
- `Kill()` spans `Projectile.cs:70642-78533`; the per-type arm chain was walked
  whole in ≤100-line windows. A hook is FUNCTIONAL if it spawns a projectile,
  runs `Damage()`, or writes/kills a tile — dust/sound-only arms belong to the
  g63-65 dust / g21-25 sound lanes and are NOT ported here.
- A hook is PORTABLE here only if its kind already has a `proj_data` row AND a
  dispatch arm on main (else it defers with its AI arm, the F3/F5 pattern).

## Verdict — six functional Kill hooks found (lane a); FOUR kept after the d78 fold

The census found six functional hooks missing for rowed+armed kinds. The
d78 FINAL adjudication (26f §1) moved the hostile EVP trio to d78; the
table records the FINAL owner per kind:

| kind | name | C# Kill arm | row / arm on main | FINAL owner / state |
| --- | --- | --- | --- | --- |
| 452 | PhantasmalEye | `:72987-73013` | `data.rs:2454` / `ai_phasmal_eye` | **d78** (`hostile_death_blast` 144→30 + `damage_evp_boxed`); g65 the dust/gore body; d77 the cue — g1 DROPPED |
| 454 | PhantasmalSphere | `:73014-73045` | `data.rs:2455` / `ai_phasmal_sphere` | **d78** (recentre 208 + verbatim EVP); g65 the body; d77 the cue — g1 DROPPED |
| 459 | ChargedBlasterOrb | `:73338-73371` (459/709 SHARED) | `data.rs:3814` / `ai_charged_blaster_orb` | **g1 KEEP** — scale≥1 → Resize 144 → `Damage()` → dust/gore rings (the :73351 order); the gate is `459 \| 709` |
| 521 | CrystalPulse | `:73292-73327` | `data.rs:3825` / `ai_crystal_pulse` | **g1 KEEP** — dust census + owner-gated 3-5× CrystalPulse2 (522) split spawn |
| 448 | SaucerMissile | `:73603-73631` | `data.rs:1960` / `ai_saucer_missile` | **d78** (recentre 112 + verbatim EVP); g65 the body; d77 the cue — g1 DROPPED |
| 80 | IceBlock | `:75533-75553` | `data.rs:2675` / `ai_ice_block` | **g1 KEEP** — ai0≥0 census + the tile-127 `WorldGen.KillTile` break |

g1's kept set lives in `crates/tr-app/src/projectile_kill_hooks_a.rs` +
the pool death hooks + the 522 row/arm (the 522's own Kill dust census
`:73271-73290` lands with it). The trio's g1 first-round code (the hooks,
`hostile_burst_colliding`, the pool's `death_burst_player_hit` and the
dispatcher arms) is DROPPED on this branch — no raw-damage EVP survives.

**709/459 dedupe cross-note (resume round, 2026-11-21):** the 459/709 arm is
SHARED (`if (this.type == 459 || this.type == 709)` `:73338`). The burst half
`:73338-73371` (resize + `Damage()` + dust/gore rings) stays THIS lane's
claim; the 709-only half `:73372-73388` (the Sky Dragon's Fury circle: the
80×80 sibling 1110 pull-in + the centre spawn + the `30·Next(2,6)` life) +
the 1110 row/arm was ported by **f20** (`task/W4-f20-proj-ai-verify-2`,
commit `01e0b1083`, census `26f` F3 — the "UNROWED, deferred" note above is
superseded by that landing). The two halves are kind-disjoint at the pool:
this lane's gate is **already widened to `459 | 709`** (executed in the
seal round — the local `MONK_STAFF_T3_ALT_SHOT_KIND` const is byte-identical
to f20's, same file position, keep-one at the g1×f20 merge; 709's row lives
on d15, so the gate is inert for 709 until then), f20's
`sky_dragons_fury_kill` owns the 709-only tail. No overlap, no double-port.

## Deferred functional hooks (kind unrowed / AI arm unported — not this lane)

These `Kill()` arms are functional but their kind has no `proj_data` row or no
dispatch arm, so they defer with the AI-arm lane (the F3/F5 "row+arm together"
pattern — porting the hook early would park the kind in `_ => {}`):

- 467/468 CultistBossFireBall (+clone) `:73046-73139` — recentre 176 + Damage;
  unrowed (the cultist fireball AI arm is unported).
- 483 SeedlerNut `:73159-73256` — the 4-8 thorn (484) homing split + the 100-px
  Damage resize; 483/484 unrowed.
- 117 BoneArrow `:74734-74751` — spawns 2× BoneShard (1124); 1124 is unrowed and
  its AI_001 per-type rider (`:54016`/`:57582`) belongs to the aiStyle-1 lane
  (d1). The 117 hook defers until 1124's row+rider land.
- 502 Meowmere `:73675-73687`, 510 ToxicFlask `:73688-73704` (the 511-513 split),
  358/406/1042 the gun streams, 171/475/505/506 the rope coils — unrowed.

## Dust/sound-only Kill arms (NOT this lane)

The remaining ~150 `Kill()` arms are dust/gore/sound-only (the 265/355/304 fang
bursts, the 731/1115/337/379 dust tails, the coin/globe arms, …). They belong to
the g63-65 projectile-dust / g16 gore / g21-25 sound lanes; this lane ports only
the FUNCTIONAL hooks above. The gore rows inside the six ported hooks are
censused as range draws (the `projectile_ai_misc::celeb2_explosion` convention)
for g16 to upgrade to the `GorePool` at its wiring wave.

## RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`)

Every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s order; the
dust/gore BODIES are skipped (no draw layer wired here) and the `Dust.NewDust*`
internal rolls go with the dust pool (the `projectile_ai_misc` convention). The
`Gore.NewGore` internal kicks DO stay — `Next(10,31)` on Y first, then
`Next(-20,21)` on X (`Gore.cs:1327-1328`), plus the explicit position
`Next(100)` pair, the `Next(61,64)` type and the two post-write `Next(-10,11)`
velocity kicks (the 7-draw `gore()` row, `celeb2_explosion` precedent). The
ring-dust `RotatedBy(2π·NextDouble())·NextDouble()` shape draws TWO
`next_double` per particle (`Utils.cs:1675-1684`). The 521 split's velocity
draws are STATE-BEARING (they place AND velocity the 522 shard) and feed the
spawn inline.

## Deviations (deliberate, recorded)

- **Hostile EVP death bursts (452/454/448) — DROPPED from g1 (26f §1).**
  The first-round port's `death_burst_player_hit` rode `hurt_with_source`
  with RAW damage (the "standing M4-HAZARD-1 deviations": no DamageVar /
  hostileDamageScaling / ×2 / BossNoCheese cooldown), and its General-lane
  immune gate would refuse the 452/454 boss blasts through spawn i-frames
  which the C# `:14799` lands. d78's `damage_evp_boxed` ports
  `Damage_EVP` (`:14773-14900`) VERBATIM (DamageVar(−owner.luck) `:14831`,
  hostileDamageScaling `:14842`, the ×2 `:14844`, the BossNoCheese-vs-
  General cooldown `:14779-14797` through `hurt_inner_cooldown`), so the
  adjudication keeps d78's state + hit and g65's dust/gore bodies; g1's
  hooks, censuses, `hostile_burst_colliding` and the pool hit fn are
  removed on this branch. No raw-damage EVP deviation remains in g1.
- **The 522 CrystalPulse2 split child** gains a row + AI arm + bounce this lane
  (it is the 521 hook's spawn target, so row+arm+hook land together — the F3
  pattern). Its `HandleMovement` bounce (`:18796-18806`) is the full ±1.0
  reflection, no penetrate spend, no kill-on-bounce (the ai[1]>40 counter kills).
- **Sound cues** (`SoundID.Item14/27/110/…`) are the sound-lane skip; the dust
  bodies are the dust-lane skip. Only the explicit draws + functional state land.
- **The 459 `Damage()` order.** The C# runs `Damage()` (`:73351`) BETWEEN the
  resize (`:73345-73347`) and the dust rings (`:73353+`); the port splits the
  arm into `charged_blaster_orb_kill` (resize + flag) and
  `charged_blaster_orb_kill_draws` (census) with the pool's `damage_pve_boxed`
  between them, so the per-hit crit/DamageVar draws stay ahead of the ring
  draws on the stream. (The dropped trio ran `Damage()` LAST, census first —
  that order lives in d78's port now.)
- **The 522 kill-tick movement skip.** The C# `Update` returns at `:17027-
  17030` when the AI killed the slot, so `HandleMovement` does NOT run on the
  ai[1] > 40 kill tick (the gravity/drag writes at `:37750-37755` still ran
  inside the AI arm); the pool gates `crystal_pulse2_move` on `active`.
- **The g65 census keep-one (26i).** g65's pushed death-dust bodies
  (`459 | 709` / `521` / `522` / `80` arms, `projectile_death_dust.rs` @
  `b0842907c`) carry the SAME explicit draws inside their bodies. At the
  g1×g65 merge the g1 census calls DROP per-kind (the bodies supersede
  them, the 26i §3 pattern); g1's STATE (the resize + Damage + the 522
  split spawns + the tile break) stays. Until that merge g1's censuses
  are the only draw source on this branch (g65's module is not here).
