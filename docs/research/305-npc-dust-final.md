# 305 — NPC dust lane FINAL (SEALED): keep-one confirmation + worm-seam census + family declaration (W4-f4, 2026-11-21)

> **RENAME CHAIN: 240 → 281 (raced, unpublished) → 283 (collides with
> f26's §7.3 reservation — `68-findframe-census-addendum-7j.md` →
> 283-findframe-census-addendum-7j) → 305 (registry-150 §7.4
> ASSIGNMENT).** The d54 collision registry
> (`150-research-number-registry.md`, round-3 EXTENSION §7.4, fetched
> at `origin/task/W4-d54-dust-sites-world`) adjudicates key 240: g58
> WINS (`cdfe3d108` 12:16:37), f4's `240-npc-dust-final.md`
> (`e7ea8b24b` 15:00:53, missed by the round-3 sweep) → **305** per
> the §7.4.3 f4 row. The executed 283 push contests f26's published
> §7.3 reservation on the same key, so 283 is NOT honored; 305 is the
> registry-assigned number (extension block 304–307 consumed by
> d50/f4/g86/g43; 308+ free). Chain recorded per the registry's
> header rule (like f11/g67/d53). ANALYSIS ref: `docs/ANALYSIS.md:39274`
> (repointed to `research/305-npc-dust-final.md` in this rename round).

> **SEALED.** W4-f4 retire round, code-only. Renumbered **240 → 283**
> (SUPERSEDED — see the rename-chain banner above; final number 305)
> per the d54 collision registry (`150-research-number-registry.md`
> round-2 §6.1 key 240 + §6.3 cutoff note): the doc was first pushed on
> 240 at 15:00:53 (`e7ea8b24b`), AFTER g58's `240-happiness-final.md`
> (12:16:37, `cdfe3d108` — the registry-confirmed winner) and g63's
> `240-proj-trail-dust-family-final.md` (14:54:20, `1c52d5ee3`, a
> post-sweep push f4 also loses to). f4 is the LAST of the three 240
> claimants; the round-2 assignment block ends at 280 and the registry
> directs post-cutoff losers to "take the next number ≥281 … after a
> fresh all-tip sweep". Fresh sweep this round: `git fetch --prune` of
> all **360** `task/W4-*` tips, `git ls-tree docs/research/` per tip
> (**21 176 rows**), leading-number grouping — keys ≥237 present at
> tips: 237–246, 264, 266, 269, 271, 273, 279, 280 (the round-3 cascade
> had not assigned f4 a number in the fetched
> `origin/task/W4-d54-dust-sites-world` research/150 — tip `72226e500`,
> round 2). The first pick **281** was raced mid-round: a pre-push
> re-fetch caught d30's `281-band-65-72-retire.md` (`ec89521fb`
> 15:42:58, executed push beats the unpublished pick) and 282 taken
> TWICE (d125 `282-d125-retire-final.md` + g48
> `282-39-item-ownership-seed.md`) — f4 takes **283**, verified
> zero-claimant at the final pre-push sweep (283–299 all free).
> Content otherwise unchanged; the §1/§5 re-verifications below were
> re-run at the current sibling tips. This doc closes the lane's
> cross-lane obligations: the e2 keep-one register, the worm-seam
> census, the d31 deferral closure, and the NPC-dust family declaration
> against the d54 bible (`131-dust-master-status.md` §8.2).

## 1. e2c-KEEP-ONE — the batch-6 MoonLord overlap, CONFIRMED + re-derived

The e2 research/64 §5 keep-one register (e2 tip `bfc081305`) assigns the
merger: **KingSlime trio 43251/43278/43365 → f4's body** (`23e313f60`,
f4 first at 05:50; e2's `king_slime_teleport_ring`/`king_slime_body_drip`
emitters + grow-ring witness supplement), **EterniaCrystal
40834/40851/40947 → f4's body** (`37d143c75`), **MoonLord core/head/eye
→ f4's full body** (`abff45cf8`; e2 b5 `31c91d90` first but partial, its
storm/smoke/fan witnesses supplement). f4 CONFIRMS all five rows — the
f4 bodies are the survivors.

**BUT the §5 "draw-equivalent" note for the MoonLord rows was checked
against e2's ACTUAL tip, not just its b5 commit — and e2's b5 fidelity
fix `6e304b960` ("C# left-to-right f32 multiply order kept verbatim")
had already re-pinned three expressions f4's b6 re-port had folded.**
The keep-one is therefore *f4's body WITH the e2-b5 multiply order
folded in* (this round, `npc.rs`):

| Site | C# expression | f4 b6 (pre-fix) | e2 b5 fix `6e304b960` | This round |
| --- | --- | --- | --- | --- |
| 41367 fan velocity | `UnitY.RotatedBy(a) * num7 * (nf*1.6f+1.6f)` (`NPC.cs:41370`) | `rot * (num7*(nf*1.6+1.6))` — scalar pre-folded | `(rot*num7)*(nf*1.6+1.6)` | **e2 form adopted** |
| 41396 inner velocity | `-Vector2.UnitY * num7 * (nf*0.9f+1.6f)` (`:41399`) | `(0, -(num7*(nf*0.9+1.6)))` | `(0,-1)*num7*(nf*0.9+1.6)` | **e2 form adopted** |
| 42222/42873 fan velocity | `Normalize(c-p) * 3.5f * (10f-num9*2f) / 10f` (`:42223`/`:42874`) | `n * (3.5*(10-2*num9)/10)` — scalar pre-folded | `((n*3.5)*(10-num9*2))/10` | **e2 form adopted** |

**Re-verified at the SEAL round** against the CURRENT sibling tips:
e2 tip `bfc081305` (its research/64 §5 register matches the table above
verbatim; `6e304b960` 05:32:48 is the multiply-order fix cited), and the
adopted forms are live in `npc.rs` (the storm fan/inner at the `:41336`
window, the head/eye fans `n * 3.5 * (10.0 - num9*2.0) / 10.0` — the
left-to-right chain). The C# rows re-read direct-path at
`decompiled/Terraria/NPC.cs:41355-41406`/`:42204-42228` and
`Utils.cs:1807-1810` (`(int)x >> 4`, both probes).

**e2-side DROP list (the merger's other half):** with f4's bodies
confirmed as the keep-one survivors, the e2-side duplicate emitters at
these sites DROP at the merge — the KingSlime trio's inline
`update_king_slime` dust arms (`1cbbcb95d`), the EterniaCrystal trio
(`31c91d90`), and the MoonLord storm/fan/eye bodies (`31c91d90` +
`6e304b960`, whose multiply ORDER f4 already absorbed). What e2 KEEPS
as supplements (per its §5): the named `king_slime_teleport_ring`/
`king_slime_body_drip` emitters + the grow-ring machine witness, the
EterniaCrystal machine witnesses, and the storm/smoke/fan witnesses
re-mirrored onto f4's bodies. f4 drops NOTHING on these sites — the
bodies are its own.

f32 multiplication is not associative — the pre-folded scalars round
differently from the C#'s left-to-right evaluation. The mirrors
(`mirror_storm`, the head-fan witness) re-derived to the same order.
Everything else verified draw-equivalent at the tips:

- **core storm** (`:41336-41403`): both bodies roll the unconditional
  nine (RandomVector2 2×NextDouble, radius NextFloat, Next(6,19), four
  NextFloats, SelectRandom Next(2)) then the gated `num4*2` fan + the
  inner loop. f4's `(int)x >> 4` tile probe (`Utils.cs:1807-1810`) is
  the exact C# form; e2's `/16` truncation differs only for negative
  coordinates (off-world either way). f4's `ceil(ai[1]/60)` iteration
  count == e2's float `while num11 < ai[1]/60` for every in-band value
  (60 < ai[1] < 580, %3==0). **f4 keeps the `>>4` probe + the ceil
  census; the smoke arm's probe ALSO fixed to `>>4` this round (it had
  kept the `/16` form — `:41415` is ToTileCoordinates too).**
- **head fan 42222**: 60..120 one / 120..180 two, ONE NextDouble angle
  each (f4: `(nd as f32) * TAU` — the C#'s f32-cast-then-multiply;
  e2: `nd * (PI as f64 * 2.0)` — f64 multiply then cast; both land the
  same f32 for the TAU product up to 1-ulp corner cases; f4's form is
  the literal C# cast order and stays), (27,59)/2 ellipse, the 16×16
  box at vector2−(8,8), (vx/2,vy/2) args, noGravity + 0.8/1.65 posts,
  `customData` dropped (both).
- **eye orbit pair 42675**: identical bodies — 2×229 at
  centre+vector5−(4,4), zero explicit draws, velocity += vector5/15,
  noGravity.
- **eye fan 42873**: the head's shape over the (30,30)/2 ellipse —
  same verdict as 42222.
- **FindFrame case-398 open burst 71938**: f4-only (e2 never reached
  FindFrame). The re-fire quirk stands as re-derived in `bcf917437`:
  the `frameCounter % 30 / 6` recompute reads row 0 for frameCounter
  1..5, so the `frame.Y == 0` gate RE-FIRES the 50×209 burst six times
  from a fresh open before counter 6 lands row 1. Both witnesses
  (single-burst park at counter 5 + the six-burst fresh open) pin it;
  the stale "fires ONCE" doc comment on `moon_core_open_burst` fixed
  this round.

## 2. f4b-worm-dust-seam — the AI_006 census, ONE unclaimed site closed

The dispatch: census the remaining NPC dust worm-seam for unclaimed
sites. Method: every `Dust.NewDust*` line inside `AI_006_Worms`
(`NPC.cs:50963-52273`) re-grepped at the f4 tip, each mapped to its
claim. **Ten sites total:**

| Site | C# block | Kind(s) | Claim | Status |
| --- | --- | --- | --- | --- |
| **50976** | `:50965-50978` leech tunnel-entry 20×5 | 117 | **f4 b7 (this round)** | **WIRED** — was the seam's one unclaimed site (the S1-E-21g note "20-gore/dust burst SKIPPED" retired) |
| 50993 | `:50984-51003` CultistDragon fade 2×228 | 454-459 | wyvern lane (`npc_ai_wyvern.rs` census `:46`) | kinds UNPORTED (no NpcKind rows) — dormant, cited |
| 51015 | `:51005-51023` BloodEel fade 2×5 | 621-623 | wyvern lane census | kinds UNPORTED — dormant, cited |
| 51026 | `:51024-51027` BloodEel ambient Next(5) 1×5 | 621-623 | wyvern lane census | kinds UNPORTED — dormant, cited |
| 51046/51054 | `:51039-51060` StardustWorm 570th-tick burst | 402 | f4 b1 (`37d143c75`) | wired |
| 51656/51662 | `:51651-51669` Crawltipede twin trail | 412 | f4 b4 (`8d5e4e7cf`) | wired |
| 52259 | `:52251-52271` EoW spawn-fade tail | 13-15 | f4 b4 (`8d5e4e7cf`) | wired |

**50976 ported verbatim** (`world_sim/npcs.rs`, the chain-init latch
block): `localAI[1]` 0→1 one-shot, `num = velocity.X < 0 ? −1 : 1`,
20× `NewDust((x−20, y−20), width+40, height+40, 5, num*8, −1)` at the
7-arg defaults (alpha 0, default colour, scale 1f), ZERO explicit
draws (type 5 hits no per-type tweak), no posts, the PRE-move box (the
C# runs it at AI_006's head before `:90498`'s integration). The queued
NPCDeath13 play keeps its after-pass drain (the audio convention). The
burst precedes the budget roll on the stream (the latch block sits
ahead of `:51310`). Two existing leech witnesses re-mirrored (the
budget-census and the tail-growth ledgers gain the 20×6
internals); 1 new draw-for-draw mirror witness (both vx sign arms +
the one-shot latch) WRITTEN NOT RUN. (The crumble witness's head is
pre-inited — `worm_inited = true` skips the latch block — so its
ledger is unchanged.)

**Adjacent windows re-checked (NOT AI_006, attribution corrections to
e2 research/64's "other-lane" table):**
- `50268/50284` — e2's table says "Wyvern fade"; actually
  `AI_005_EaterOfSouls`' **BloodSquid 619** day-flee burst (35×5 at
  alpha==255 + 2×5/tick, `:50254-50291`). Kind 619 UNPORTED — dormant,
  the hover lane's (d22/f4-b5 family) when the kind lands.
- `50692` — the hover gate's **619 unconditional arm** (1×5, vx*0.2/
  vy*0.2, alpha 100, velocity*=0.5 post, NO Next(20) gate — the C#
  `:50690-50694` splits BEFORE the gate). f4 b5's `hover_dust_gate`
  models the bounced/excluded arms only; 619 has no kind, so the arm
  is dormant — cited for the kind wave, not a live gap.
- `49801` Destroyer fade — f4 b3 wired (`0b7b14e29`), AI_037 not
  AI_006 (`is_worm()` excludes the family).
- `52496/52713` — AI_002 FloatingEye dust (the eye lane's `EyeDust`
  seam), NOT the worm pass.
- FindFrame worm cases (`case 10` `:72048`, the `:77945+` frame-count
  list incl. 7/10/13/87/127-131) carry ZERO dust — the only FindFrame
  dust in the boss band is f4's own 71938.

**Worm-seam verdict: ZERO unclaimed LIVE sites remain.** The five
dormant sites (50268/50284/50692/50993/51015/51026 — six lines, five
blocks) all gate on unported kinds (619/454-459/621-623); their claims
are cited above for the kind waves.

## 3. d31-deferred — the 5-site claim CLOSED

d31's ANALYSIS ("The aiStyle 75-83 axis census", tip `0487ea324`)
deferred the dust-GATED draws to the dust lane: the drama fan
`:41365-41373` + inner `:41396-41401`, the head's 229 fan `:42221`, the
eye's dust pairs (`:42675`/`:42873`). f4 b6 (`abff45cf8`) claimed and
ported all five sites (the core storm counts as one window, two spawn
sites); this round's §1 re-derivation completes the closure. d31's
machine bodies (the axis arms, the ritual, the census witnesses) stay
its own — no d31 file touched. The `getGoodWorld` 1021 fan
(`:42267-42276`) stays d31's documented deferral (a projectile spawn,
not dust; needs the runtime seed flag).

**Re-read at the SEAL round**: d31 tip `0487ea324` ANALYSIS — the
"Verdict per block" row ("Drama dust storm `:41336-41403` … the
dust-GATED fan draws stay deferred with the dust engine") and the
"Deferred" paragraph (the drama fan `:41365-41373` + inner
`:41396-41401`, the head's 229 fan `:42221`, the eye's dust pairs) are
UNCHANGED at the tip — they are d31-lane reads, and this §3 is the
dust-lane closure receipt for them (the merger may annotate d31's rows
"closed by f4 research/305"; f4 does not edit d31's branch). d31's
9+3·inner unconditional draw census and f4's storm port consume the
SAME draws in the SAME order — the two ledgers agree.

## 4. npc-dust-family declaration (the d54 bible §8.2 row, f4 slice)

The bible's NPC.cs family: **927 NewDust* + 5 CloneDust = 932** (836 ND
+ 16 NDP + 75 NDD; HitEffect window 629+2). The f4 lane's FINAL
contribution, at tip `b62d6c2a3` + this round:

| f4 batch | Sites | Count |
| --- | --- | --- |
| b1 `37d143c75` | Deerclops 44037, EterniaCrystal 40834/40851/40947, Fishron 49029/49181/49449/49661, StardustWorm 51046/51054 | 10 |
| b2 `23e313f60` | KingSlime 43251/43278/43365 | 3 |
| b3 `0b7b14e29` | Destroyer fade 49801 | 1 |
| b4 `8d5e4e7cf` | Crawltipede 51656/51662, EoW fade 52259 | 3 |
| b5 `87da911e4` | hover gate 50702/50710 | 2 |
| b6 `abff45cf8` | core storm 41367/41396, head fan 42222, eye pair 42675 + fan 42873, open burst 71938 | 6 |
| b7 (this round) | leech entry 50976 | 1 |
| **total** | | **26 sites** |

**NPC-dust-family bible (the b1–b7 seal):** the table above is the
COMPLETE f4 slice — batches 1–7, 26 sites, every row cited to its
landing commit and re-verifiable at this branch's tip. Cross-checks:
e2's research/64 §2 AI-window table lists every f4 row with the same
site ids + commits (tip `bfc081305`); the d54 bible §8.2's NPC 927+5
family count treats these as claimed; the wave-E gore doc
(research/211) is disjoint (gore, not dust).

Plus wave E's 17 gore sites (research/211, renumbered 71 → 211 per
registry-150 §1 — refs re-verified at this SEAL round: the ONLY
carriers are `npc_hit_effect_e.rs` header comments ×2 and
`docs/ANALYSIS.md`, all pointing at 211; zero stale `71-npc-gore-wave`
refs anywhere on the branch). The f4 keep-one survivors per §1:
KingSlime trio + EterniaCrystal trio + the five MoonLord windows (with
the e2-b5 multiply order). The NPC family's remaining unclaimed
remainder is the bible's own: the buff-VFX producers (>90322) ride
their buff lanes (c16-c18/d71-d74), the 2 HitEffect CloneDust ride
g62's ledger — **nothing owed by f4**.

**SEAL declaration:** with this doc's four closures confirmed
(§1 keep-one + multiply-order adoption, §2 worm-seam ZERO unclaimed
live sites, §3 d31 deferral closed, §4 the 26-site family slice), the
W4-f4 dust lane is **SEALED** — no further dust batches are owed, no
open claims remain, and the merger takes this branch's tip as the
lane's FINAL state. Any later f4 push is doc-integrity only (this
renumber round included).

## 5. Verification record

Code-only round: **no cargo/rustc/rustfmt/tests/dotnet/graphify
invoked** (the lane directive). Witnesses WRITTEN NOT RUN. The C#
re-reads cited above are direct-path reads of
`decompiled/Terraria/NPC.cs`/`Utils.cs` (no junction recursion). The
e2/d31/d54 comparisons are `git show <tip>:<path>` reads at the exact
tips named.

**Original retire-round scan (240 pick, SUPERSEDED):** `git ls-tree`
over the then-355 remote `task/W4-*` tips found 237–239 taken and 240
free — but g58's 143→240 self-rename (`cdfe3d108` 12:16:37) was already
at its tip under a different FILENAME the scan's "240 free" check
missed, and the registry round 2 (published 14:47:50, after that scan)
adjudicated 240 to g58. This SEAL round's fresh sweep (360 tips,
21 176 `docs/research/` rows, fetch `--prune`): 240 has THREE claimants
(g58 12:16:37 winner / g63 14:54:20 / f4 15:00:53 last) — f4 renames
240 → **283** (the fetched d54 research/150 at `72226e500` carries no
f4 assignment, so the §6.3 self-adjudication rule applies: first free
≥281 after a fresh sweep). The first pick 281 was raced: the pre-push
re-fetch caught d30's `281-band-65-72-retire.md` (`ec89521fb`
15:42:58 — an executed push beats the unpublished pick) and 282 taken
twice (d125 retire-final + g48 item-ownership-seed); **283 verified
zero-claimant** at the final pre-push sweep (283–299 free). 243
(g58+d50) and 244 (d53+f39+g48+g86) are live post-cutoff collisions
for OTHER lanes — not f4's to adjudicate; noted for the round-3
cascade. **SUPERSEDED by registry-150 §7.4:** 283 collides with f26's
published §7.3 reservation (`68-findframe-census-addendum-7j.md` →
283); the §7.4.3 f4 row assigns **305** (chain 240 → 281-raced →
283-collides → 305).

**SEAL-round re-verifications (this commit):**
- 71→211 rename refs: `git grep -n 71-npc-gore-wave-e-claims HEAD` →
  EMPTY; `211-npc-gore-wave-e-claims` carried only by
  `npc_hit_effect_e.rs:5`/`:57` + `docs/ANALYSIS.md:39255` — all
  correct.
- e2-b5 multiply order live in `npc.rs`: storm fan
  `Vec2::new(-sn, cs) * num7 * (nf*1.6+1.6)`, inner
  `Vec2::new(0.0,-1.0) * num7 * (nf*0.9+1.6)`, head/eye fans
  `n * 3.5 * (10.0 - num9*2.0) / 10.0`; both tile probes
  `(x as i32) >> 4` (storm `:41346` + smoke `:41415`).
- case-398 comment: `moon_core_open_burst` doc carries the six-burst
  re-fire quirk (no "fires ONCE" text remains); both witnesses present
  (`w4_f4_the_core_open_burst_refires_six_times` asserts 6 bursts).
- leech 50976: the latch block in `world_sim/npcs.rs` spawns 20×5
  pre-move ahead of the budget roll; the two leech ledger witnesses
  carry the 20×6-internals oracle replay.
- C# spot re-reads: `NPC.cs:41355-41406` (the nine draws + fan/inner
  velocities), `:42204-42228` (the head fan's
  `Normalize(center-vector2) * 3.5f * (10f - num9*2f) / 10f`),
  `Utils.cs:1807-1810` (`ToTileCoordinates` = `(int)x >> 4`).
- d31 tip `0487ea324` / e2 tip `bfc081305` / d54 tip `72226e500`
  re-fetched and read at those exact tips (the §1/§3 receipts above).