# 364 — Projectile-gore-family FINAL + SEAL (W4-g16, task/W4-g16-gore-projectile-sites)

> **Number chain: 247 (invalid self-pick) → 360 → 364.** The first draft
> self-picked 247 — INVALID (247 = d10's §8 reservation, executed
> `72431e61f`), renumbered to 360 pre-publish (`1254ff5e0`); 360 collided —
> g10's `360-g10-pets-final` (`7414ba339`, 17:31) was first and KEEPS 360
> per registry-150 §9 UNIFIED FINAL (d54 tip `b4c94432c`), row 364:
> g16 `360-proj-gore-family-final` (`1254ff5e0` 17:40:42, third 360
> claimant) → **364** (this rename; docs-only, no content change).

Status: **SEALED** (2026-11-21, code-only — no cargo/tests/rustfmt/graphify
invoked). Number per registry-150 §9 UNIFIED FINAL (d54 tip `b4c94432c`,
§9.2 row 364): this doc was sealed as **360** under the then-current §8.7
FROZEN table (d54 tip `a15cb634b` — the 151-236 + 246-351 blocks consumed
(191 assignments), NO self-picks below the free floor — **new docs take
≥360 after a registry check**; a fresh 364-ref `ls-tree docs/research/`
sweep at this seal found max seated **346** (g65's bible), nothing ≥352 →
**360** claimed). §9 re-assigned it to **364** (g10 first-add KEEPS 360).
The first draft of this doc self-picked 247 — INVALID (247 = d10's §8
reservation, executed `72431e61f`), renumbered before publish. Witnesses WRITTEN NOT
RUN. C# cites are `decompiled/Terraria/Projectile.cs` (stable), read by
DIRECT file path in ≤100-line windows (the junction is never recursed).
Rust cites pin g16's tip **`72e4cae20`** and g65's pushed tip
**`37ec2f3aa`** (module lines 1-7480 byte-identical to `f6e07bb90` — the
seal delta is +19 witness lines appended at the tail) unless a branch tip
is named.

Companions: research/111 (`b68eac29`, the f5 GORE SEAL — the family census +
gate-command + §10.1 bible, CONSUMED here), research/26i (`f6e07bb90`, the
g65 Kill keep-one receipt) + research/346 (`37ec2f3aa`, the g65 death-dust
family bible — this doc is g16's side of the same adjudication; the 346
drop-hunks 1-17 and this doc's §3.2 gore-stub drops are DISJOINT hunk sets
on the same module).

## 1. The 165-site census re-verification (git log census)

The lane's commit census at the seal tip: **27 commits** on
`origin/main..HEAD` (merge-base `a725acb04`). Waves A-O in source order
(`87db6b544` → `083fdae80`), the two gap fills (`f2d4092d3` the `:24037`
type-9 trail, `f08823aea` the `:36331` crystal burst + `:75871-75883` 240
corners), the wave-O docs fix (`613497d82` — the census tail is 13 call sites
152-164; the big-bomb block's 4 sites run twice = 8 spawn events), then the
wiring: batch 1 `1deeb8ca8` (the GorePool seam), batch 2 `36e03444a` (the
AI-side sites), the 284 no-double-feed fix **`f1c60ffe9`** (the census pin the
f5 seal §10.3 recorded as g16's tip), batch 3 `69eaf77fb` + batch 4
`a05959ec0` (the Kill gore arms), the wiring docs `4e589db91`, batch 5
`43a99ff10` (the 1025 burst + the wave-O bomb rows), the 1025 latch fix
`1392d27f4`, the 955 witness AI-order fix `72e4cae20`, then this seal.

**Coverage proof (this seal's re-run)**: the direct grep of
`Projectile.cs` finds exactly **165** `Gore.NewGore(Perfect|Direct)?\(` sites
(line list re-derived, not copied). Every line is cited by a
`projectile_gore_*.rs` module: 159 by exact `:NNNNN` cites, the remaining 6
(`:75871/:75875/:75879/:75883` + `:77678/:77682`) by range cites
(`gore_j.rs:138-139`'s `75871-75886` across a doc-comment line break;
`gore_m.rs:13`'s `` `:77674`-`:77686` `` table row) — **165/165, zero
uncited**. The 15 modules carry 87 pub site fns + 73 module witnesses
(67 wave + this seal's 6 compose witnesses, §3) + 14 wiring witnesses in
`projectile.rs` — **87 g16 witnesses total**.

**The family gate-command re-run (111 §10.2, verbatim)**:

```
NPC : 1345   Projectile : 165   WorldGen : 36   Player : 32   Main : 0
```

Σ **1578** — matches the seal. The f5 window gate re-run: **853**. No drift.

## 2. The f5-gore-seal consumption (b68eac29)

111 §10.1's GORE-total-1578 declaration is consumed verbatim: NPC 1345 =
HitEffect 1304 (**d55 26 + e5 40 + f10-legit 385 + f5 853**) + 41 other;
**Proj 165 = this lane**; WorldGen 36 + Player 32 = g17; Main 0. g16's
obligations under that seal:

* The 165-site port — DONE (waves A-O, §1).
* The wiring (the `UpdateProjectiles` slot dispatch) — DONE for every rowed
  host (batches 1-5); the unrowed hosts are cited deferrals (the ANALYSIS
  "Cited deferrals" list; a kind with no `proj_data` row never spawns, so
  its arm cannot fire — the `npc_hit_effect_a.rs` precedent).
* The f10-overrun delete / the g62 hit-half assignments — NOT g16's (NPC-file
  lanes); this lane's modules carry zero NPC-file gore refs.
* The g16 tip pin in 111 §10.3 (`f1c60ffe9`) is superseded by this seal's
  §6 pin table (the lane advanced through wiring batches 3-5 after that
  seal's fetch).

## 3. The g65-Kill-309 compose (proj-death-dust vs gore — DISJOINT bodies)

g65 (`task/W4-g65-proj-death-dust`, tip `37ec2f3aa`, module
`projectile_death_dust.rs` 7502 lines) ports the Kill death-DUST bursts.
**The owed-pool label's "309" is the stale batch-28 census** (g65's 346
bible §3 correction): the FINAL drain is **241 match arms → 316 distinct
kinds** — re-derived independently at this seal (every arm header
`:540-4680` ranges-expanded, single-digit and mixed `121..=126 | 597` arms
included; the roster matches 346 §3 kind-for-kind, zero diff both ways).
The 26i "317" was a +1 miscount, the ANALYSIS "309" the batch-28 figure —
**316 is the number**, 313 post-merge (the 358/406/1042 d6 drops, 346 §2b).
The disjointness receipt, verified by grep at both tips:

* **g65 carries ZERO gore bodies**: `GorePool|new_gore|.gore` — **0 hits** in
  `projectile_death_dust.rs`. Where a gore draw interleaves with dust, g65
  consumed the DRAWS ONLY as census stubs (`gore_census` :4756 + the inline
  `let _ =` loops), bodies explicitly "stay the gore lane's" (the beenade
  precedent, its :4755 doc).
* **g16 carries ZERO dust bodies**: `new_dust|DustPool|dust::` — **0 hits**
  across the 15 `projectile_gore_*.rs` modules. The dust loops beside every
  gore site are the dust lanes' documented skip (the module RNG contract).

So the two lanes' BODIES are disjoint — both merge whole. The overlap is
draw-only: g65's stubs and g16's real spawns consume the SAME stream values
for the same kinds. Keeping both double-rolls; the merge executes the
drop-table below (g65's stub draws DROP, g16's bodies KEEP — the g65-wins
convention inverted for the gore half, exactly as 26i §2 assigned "g65 owns
the dust/gore BODIES when pushed" for dust and this doc answers for gore).

### 3.1 Draw-equivalence (verified by C# re-read + line-by-line stub compare)

The `gore_census(rng, count, scatter, lift)` stub (:4756-4773) replays per
gore: `Next(100)` X/Y pair → `Next(61,64)` type → kicks `Next(10,31)` Y then
`Next(-20,21)` X (`Gore.cs:1327-1328`) → [scatter: `RotatedByRandom(π)` TWO
`NextDouble` + `NextDouble` radius] → [lift: `Next(2)` gate] → two
`Next(-10,11)` nudges. The g16 bodies draw the identical sequence:

| g65 stub call (kind) | C# gore window | g16 body (keep-one) | verdict |
| --- | --- | --- | --- |
| `:269` dd2_explosion_dust, `gore_census(ngore,true,true)` (664/666/668/706) | `:71888-71900` | `gore_f::dd2_explosion_gores` (count 4/8, scatter+lift+the −16 shift) | EQUAL — the −16/−30 are body writes, no draws |
| `:2056` 645, `(1,false,false)` | `:72118-72124` | `gore_f::kill645_gore` (celeb2 row) | EQUAL |
| `:2152` 617, `(2,true,false)` | `:72319-72326` | `gore_f::kill608_617_gores` → `gore_b::fire_explosion_gores` | EQUAL |
| `:2394` 452, `(1,false,false)` | `:73004-73010` | `gore_g::kill_row_gores` ×1 | EQUAL |
| `:2420` 454, `(3,false,false)` | `:73037-73043` | `kill_row_gores` ×3 | EQUAL |
| `:2458` 467, `(2,true,false)` | `:73084-73091` | `gore_g::kill467_468_gores` | EQUAL |
| `:2492` 468, `(2,true,false)` | `:73131-73138` | same keep-one | EQUAL |
| `:2599` 459\|709, `(ngore,false,false)` (2 at scale≥1 else 0) | `:73365-73371` | `kill_row_gores` ×2 gated | EQUAL (the 709 no-row deferral stands) |
| `:2679` 448, `(1,false,false)` | `:73623-73629` | `gore_h::kill448_616_424_gores` ×1 | EQUAL |
| `:2701` 616, `(1,false,false)` | `:73666-73672` | same ×1 | EQUAL (host unrowed, deferred) |
| `:2747` 424..=426, `(2,false,false)` | `:73841-73847` | same ×2 | EQUAL (host unrowed, deferred) |

The inline stubs beyond `gore_census` (same method, direct compare):

| g65 inline stub (arm line) | kind(s) | g16 body | verdict |
| --- | --- | --- | --- |
| `:4409-4413` | 681 corner 4×3 | `gore_f::goblin_bomb_gores` | EQUAL — sign postlude draws nothing |
| `:3368-3372` | 240 corner 4×3 | `gore_j::kill240_corner_gores` | EQUAL |
| `:3315-3319` | 281 corner 4×3 (+the `:3297-3302` head 2×4 two-roll velocities) | `gore_j::corner_gores`(1.0) + `tombstone_head_gores` | EQUAL (Kill host deferred; AI landing wired) |
| `:3346-3350` | 162 corner 4×3 | `corner_gores`(1.5) | EQUAL (same deferral) |
| `:2895/:2897` → `confetti_burst` `:4968-4979` | 178 ×40 / 289 ×15 | `gore_h::confetti_gores` | EQUAL — type+kicks+the ADD-then-MULTIPLY ladder, ten ranges |
| `:3803-3813` | 284 ×5 | `gore_k::kill284_confetti` (`confetti_gores_vel` at −vel×0.3) | EQUAL — the `f1c60ffe9` no-double-feed fix is this seam |
| `:3833-3837` | 286 ×1 | `gore_k::kill286_gore` | EQUAL — NO Next(100) pair (fixed pos−10) |
| `:3857-3863` | 41 ×2 | `gore_k::kill41_gores` | EQUAL — ×0.4 + the ×0.1 nudges (not ×0.05) |
| `:3896-3901` | 955 star ×7 | `gore_j::kill12_star_gores` → `gore_i::star_kill_gores` | EQUAL — ONE NextFloat CircularEdge + Next(8) pick + kicks; g65 folded the screen gate TRUE, g16 gates on the real padded screen (the drop makes off-screen C#-correct: zero rows) |
| `:3742-3747` + `:3752-3756` | 12 star ×7 + damage<100 ×3 | same + `kill12_low_damage_gores` | EQUAL (host unrowed, deferred) |
| `:3696-3702` | 9 ×3 / 503 ×2 | `gore_j::kill9_503_gores` | **503 DIVERGES** — see §3.3 |
| `:3943-3947` | 246 ×1 | `gore_l::kill246_gore` → `gore_g::kill483_gore` | EQUAL — the INTEGER Next(-1,2) nudges (host unrowed) |
| `:3966-3970` | 249 ×1 | `gore_l::kill249_gore` | EQUAL (host unrowed) |

Six compose witnesses pin the SIX stub shapes against the g16 bodies
(`projectile_gore_o.rs`, `w4_g16_seal_*` ×6 — the plain/scatter+lift/
scatter-only/confetti/corner/star rows, stream-peek equality + spawn counts).
WRITTEN NOT RUN.

### 3.2 The merger drop-table (g65 side; execute at the g16×g65 merge)

At g65 tip `37ec2f3aa` (`projectile_death_dust.rs` — lines 1-7480
byte-identical to `f6e07bb90`, the pins hold at both), DROP the gore-draw
stubs for every kind whose g16 body is WIRED (the body draws the same
values on the same pass stream); KEEP the stubs for the DEFERRED hosts
(g16's arm cannot fire — the stub is the only stream consumer):

**DROP (g16 wired):**
1. `:269` — the `gore_census(rng, ngore, true, true)` call in
   `dd2_explosion_dust` (664/666/668; **706 CAVEAT** below).
2. `:2056` — the 645 arm's `gore_census(1,false,false)`.
3. `:2152` — the 617 arm's `gore_census(2,true,false)`.
4. `:2394` / `:2420` — the 452/454 arms' census calls.
5. `:2599` — the 459|709 census: drop for **459 only** (`s.kind == 459`
   guard or arm split); the 709 half KEEPS its 2-gore census (unrowed).
6. `:2679` — the 448 census call.
7. `:2895` — the 178 `confetti_burst(…, 85, 40)`: pass `gores = 0` (the dust
   half stays; the helper's gore loop draws nothing at 0). The 289 call
   `:2897` KEEPS 15 (unrowed).
8. `:3368-3372` — the 240 corner stub loop.
9. `:3696-3702` — the 9|503 stub: drop for **9 only**; 503 KEEPS (unrowed +
   the §3.3 divergence).
10. `:3803-3813` / `:3833-3837` / `:3857-3863` — the 284/286/41 stub loops.
11. `:3896-3901` — the 955 star stub loop.
12. `:4409-4413` — the 681 corner stub loop.

**KEEP (g16 deferred hosts — the stub is the stream's only consumer until
the host's row lands):** 856 `:1762` (gore excluded already — the arm draws
dust only, nothing to drop even when wired), 590 `:2280-2284`, 676
`:2330-2341`, 501 `:2359-2365`, 467 `:2458`, 468 `:2492`, 616 `:2701`,
424-426 `:2747`, 289 `:2897`, 723 `:3025-3029`, 724-family
`:3042-3046`, 1053 `:3069-3073`, 1020|1105 `:3115-3121`, 1085
`:3135-3138`, 281 `:3297-3302`+`:3315-3319`, 162 `:3346-3350`, 712
`:3508-3512`, 370|371 `:3538-3550`, 1023 `:3561-3567`, 1078
`:3655-3664`, 12 `:3742-3747`+`:3752-3756`, 246 `:3943-3947`, 249
`:3966-3970`, the 709 half of `:2599`, the 503 half of `:3696`.

**706 CAVEAT (the one live mismatch found by this compose):** the C# arm
`:71821` covers 664|666|668|**706**; g65's drain arm `:1944` covers all
four; g16's `gore_death_arms` covers only `FLAME_BURST_SHOT_1|2|3`
(664/666/668 — no 706 const exists in `projectile/data.rs`). If 706 has a
spawnable row at merge, dropping `:269` whole loses its 4/8-gore census.
Merger instruction: re-check 706's row status at the merge tip; if rowed,
either extend g16's match arm with the 706 kind (one-line, the same body)
or gate the drop on `s.kind != 706`. If unrowed (expected — the ANALYSIS
deferral list's precedent), the whole drop is safe.

**Order note (not a drop):** g65 drains at the pass TAIL
(`world_sim/projectiles.rs:419-429` @ f6e07bb90) while g16's bodies fire
inline mid-walk. After the drops, each gore draw has exactly ONE consumer
(g16, inline — the C# position); the surviving dust draws keep the
pass-tail convention. Per-death slot order is preserved on both sides;
the cross-kind interleave difference is the already-recorded pass-tail
deviation (the gallop-dust precedent), not new drift.

**Compose with 346's own drop-hunks:** the g65 bible's 17 hunks (the d77/g3
census-call drops in `projectile.rs`/`projectile_ai_misc.rs` + the §2b d6
gun-smoke drops :2786-2794/:4834-4885/:6616-6634 in the death-dust module)
touch DIFFERENT lines than this §3.2 table — the two hunk sets compose
without overlap; the 346 post-merge 313-kind roster is unaffected (these
drops remove DRAW STUBS inside surviving arms, not arms).

### 3.3 The 503 divergence (recorded, g65-side fix owed)

C# `:75727-75733`: the loop ALWAYS draws `num726 = Main.rand.Next(16,18)`;
the `type == 503` arm then OVERRIDES the value to 16 — the draw is spent
either way. g16's `kill9_503_gores` keeps the draw (`gore_j.rs`, the wave-J
witness pin). g65's stub `:3697` gates the draw on `s.kind != 503` — WRONG
for 503 (one draw short per gore, ×2). The 503 Kill host is unrowed on both
sides today (no live drift), but the stub is the future stream consumer:
**g65 owes the one-line fix** (draw unconditionally, override after — the
g16 shape) before the 503 row lands. Published here for the g65 lane; not
patched on this branch (another lane's module).

## 4. The proj-gore-family bible (FINAL, one paragraph)

**The Projectile.cs `Gore.NewGore*` family is 165 sites across three axes,
every site ported and every rowed host wired, ZERO unowned: the AI axis
(waves A-D + the gap fills — enchant/superstar/tombstone-landing/trails/
lava-bounce/drill/1025-burst/volcano/fire-explosion/pirates/rope/toilet/
celeb2/ground-explosion/chum/ai026 — `projectile_gore_a..d.rs`, wired at
their C# call positions in the aiStyle dispatches + the census→pool
upgrades), the Kill axis (waves E-L — the per-kind death arms
`gore_death_arms` + the special-arrow/bullet/thrown hooks, kind-gated
once-per-death, the resize/Damage walks in C# order, the unrowed hosts
cited as deferrals), and the Kill_Bombs axis (waves M-O —
`rocket_kill_census`'s six families upgraded census→pool +
`explode_bomb`'s bomb-corner/dynamite rows on the PASS stream, the frozen
zero-rng Damage walk untouched — no double-feed). The bodies live in 15
modules (`projectile_gore_a.rs` .. `_o.rs`, 87 pub fns), ride the caller's
`UpdateProjectiles` stream through `&mut dyn DropRng` (the stream-exact
`next_max`/`next_float_direction` defaults), spawn into the App-threaded
`GorePool` (`set/take_gore_pool`, the padded-screen + anniversary seams),
and are DRAW-DISJOINT from the dust lanes by contract: g65's death-dust
drain (241 arms, 316 kinds — 313 post-merge per its 346 bible) owns the
dust bodies + the gore-draw census stubs that §3.2's drop-table retires
for every wired kind at merge; the NPC-file
gore family (1345 = d55/e5/f10/f5) and the WorldGen/Player family (68, g17)
are the sibling lanes of the 1578-total (research/111 §10.1); the d77/g3
cue lanes and the d78 state lane draw zero gore (26i §2). All 87 witnesses
are WRITTEN NOT RUN (code-only) and owed one compile window before merge.**

Gate-command (the merge-time re-verification): the 111 §10.2 five-file grep
(Σ1578, Projectile 165) + this doc's §1 coverage check (165/165 cited) +
`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
(judge by the `test result: ok` lines) once the compile window opens.

## 5. Verification performed (code-only)

No cargo/tests/rustfmt/graphify invoked (lane directive). This seal
re-derived, by direct read/grep/script:

* The 5-file gate-command re-run: 1345/165/36/32/0 = **1578**; the f5
  window re-run: **853** (both match 111 §10.2's seal-time values).
* The 165-line coverage cross-check: every grep line cited by a module
  (159 exact + 6 range-cited, §1) — zero uncited.
* The 27-commit lane census (`git log origin/main..HEAD`, §1).
* g65's drain arm re-count: **241 arms → 316 distinct kinds** (the full
  range expansion incl. the single-digit + mixed arms, §3) — the 346
  bible's roster matched kind-for-kind, zero diff both ways; the
  owed-pool "309" is the stale batch-28 figure, 26i's "317" a +1 miscount.
* The disjointness greps: g65 module 0 gore-body refs; g16 modules 0 dust
  refs (§3).
* The 11 `gore_census` call sites + the inline stub loops (681/240/281/162
  corners + heads, the confetti_burst helper + 284, 286, 41, 955, 12 ×2,
  9|503, 246, 249 — and the KEEP-side 590/676/501/723/724-family/1053/
  1020|1105/1085/712/370|371/1023/1078) mapped to
  their kind arms and C# windows; the draw-equivalence verified
  line-by-line against the g16 body fns (§3.1), the C# windows re-read
  (`:71888-71900`, `:72118-72124`, `:72319-72326`, `:72988-73012`,
  `:73084-73091`, `:73819-73860`, `:75681-75747`, `:75975-76034`,
  `:78329-78352`, `:78412-78444`).
* The two divergences found: the 706 arm-coverage caveat (§3.2) and the
  503 stub draw-gate (§3.3) — both published, neither live today.
* The registry check: registry-150 LATEST at seal time (d54 `a15cb634b`
  §8.7 FROZEN) — the 151-351 blocks consumed, new docs ≥360; a fresh
  364-ref ls-tree sweep found max seated 346, nothing ≥352 → **360**
  claimed (the first draft's 247 self-pick was INVALID — d10's §8
  reservation — and renumbered before publish). §9 UNIFIED FINAL (d54
  `b4c94432c`, row 364) re-assigned the number: g10's 360 first-add
  (`7414ba339` 17:31) KEEPS 360; this doc (17:40:42, third 360 claimant)
  → **364** — chain 247-invalid-self → 360 → 364.
* The 6 compose witnesses appended to `projectile_gore_o.rs` (§3.1) —
  WRITTEN NOT RUN.

## 6. Seal-time lane-tip pins (this seal's fetch)

| lane | branch | tip |
| --- | --- | --- |
| g16 (this) | `task/W4-g16-gore-projectile-sites` | `72e4cae20` + this seal commit |
| g65 | `task/W4-g65-proj-death-dust` | `37ec2f3aa` (the 346 bible; module lines 1-7480 byte-identical to `f6e07bb90`) |
| f5 (the family seal) | `task/W4-f5-dust-proj-3` | `b68eac297` |
| f10 (ALIVE) | `task/W4-f10-gore-sites-3` | `afea4c0fc` (advanced past 111's b157 `d197124d1` — RE-CENSUS-AT-MERGE stands) |
| g17 | `task/W4-g17-gore-world-sites` | `3dec710dc` (advanced past 111's `f0fe1bbdd`) |
| d55 | `task/W4-d55-gore-table-sites` | `ec78d1524` (111 §10.3, unchanged) |
| e5 | `task/W4-e5-gore-sites-2` | `96be85530` (advanced past 111's `a62a32ce6`; the 351 bible) |
| d78 (state co-lane) | `task/W4-d78-ui-sounds-menus` | `7ea64ff75` (the 26f §6 SEAL addenda; `fd59c51a` adjudication stands) |
| d77 (cue co-lane) | `task/W4-d77-proj-sounds-3` | `5f10560b2` (26i, unchanged) |

The g65/f10 tips are ALIVE — the §3.2 drop-table's line pins are at the
named tips and MUST be re-pinned at the merge-time tips (the 111 §8
re-census precedent). The kind assignments and draw-equivalence verdicts
are tip-stable (the C# is frozen).

**364 SEALED (sealed as 360; renumbered per registry §9 row 364). The projectile-gore family's census, wiring, compose
drop-table and bible are FINAL; the moving parts are the two published
caveats (706 row-status, 503 stub gate) and the merge-time re-pin.**