# 308 — DUST-MAIN-FAMILY BIBLE (W4-g13 SEAL): the git-log census, the d54×g13 keep-one, the 94/94 AddLight census, the registry renames EXECUTED

Status: SEAL (W4-g13, code-only — one census witness WRITTEN NOT RUN in
`dust_lights_b.rs`; the four registry-150 renames EXECUTED with every ref
re-pointed; no cargo/rustc/rustfmt/tests/graphify invoked; no merge, no
primary-checkout write). Base `eef0202cc` (the LIGHTS RETIRE phase-4
FINAL-FINAL declaration). Number 308 taken per registry-150 §7.4 ("Numbers
308+ are FREE") + a fresh all-tip `ls-tree docs/research/` sweep at this
round: 361 `task/W4-*` origin tips, ZERO claimants ≥308 at scan time (the
281/282 provisional picks raced six 15:53-16:07 pushes — c25/snail/d106/
proj-81-90/convert-tables/f39 — exactly the §6.3 racing-pick failure mode;
308 self-adjudicated per the cutoff rule).

Sibling probes this round (READ-ONLY, direct paths, no junction recursion):
d54 origin `0426245f2` (registry §7.4 extension; `dust_table.rs` blob
UNCHANGED `58305d0a8`), d50 origin `7474ce3d9` (the RETIRE FINAL **PUSHED** —
the resume the g13 receipt predicted is EXECUTED, §4), g13 tip `eef0202cc`
clean before this seal.

## 1. The git-log census — G1-52-lights-CLOSED, Main-COMPLETE, FINAL-FINAL

`git log a725acb04..eef0202cc` = **18 commits**, the whole lane. The three
seal-anchor declarations, verified by hash + timestamp (`%cI`):

| Declaration | Hash | Time (+02) | Evidence |
| ----------- | ---- | ---------- | -------- |
| **G1 — the 52 second-chain lights CLOSED** | `a234107ac` (port) + `441f6120f` (census) | 08:01:10 / 08:01:44 | `dust_lights_b.rs` 52-arm collector WIRED via `feed_dust_chain_lights` in `app/frame/draw.rs` after the FIRST-ladder feed; 42+52 = 94; research/206 (was 69) §1-3 |
| **Main.cs dust family COMPLETE** | `4444e1011` | 09:01:54 | research/202 (was 68) §5: 23-site spawn census ZERO unclaimed at g11 `9a9ac1fd` / g12 `f24dd76a`; non-spawn machinery closed (DrawCapture cull `1af798478`); the g11 snowing residual SUPERSEDED (d130 snow.rs) |
| **FINAL-FINAL RETIRED** | `eef0202cc` | 09:51:17 | the retire chain keep-one `8fd62bd7d` / wire-hunks `44eb86deb` / d50 dead-clean probe `f9566e5b4`; research/202 §6 + 206 §9 + 210 §8 |

The full 18 (oldest first): `dca09bed9` sites 17-23 port → `150e7204a`
lavaBubbles increment → `4c5499047` DrawnThisUpdate relocation →
`35404f8a8` draw-call feeds → `909397b7a` wiring docs → `4063f0eed`
item-sparkle 16px-box fix → `15fd99c90` witness fixes → `1af798478`
DrawCapture cull → `5ea44967b` sweep docs → `a234107ac` + `441f6120f`
52 lights → `6a2f68304` d50 receipt → `daf7b7804` GetAlpha 157/327 →
`4444e1011` Main COMPLETE → `8fd62bd7d` keep-one → `44eb86deb` wire-hunks →
`f9566e5b4` d50 probe → `eef0202cc` FINAL-FINAL.

**Census verdict: every claim in the three declarations re-verified at
this seal against the live blobs/C# (not commit messages).** The lane's
merge surface: `main_dust.rs` (three-way union owed, research/116 §6),
`dust_lights_b.rs`, `dust_table.rs` (keep-one blob), `particles.rs` (the
inline lavaBubbles increment — d50's strip already made it canonical),
`app/frame/draw.rs` (feeds + DrawnThisUpdate), docs/anchors.

## 2. The d54-bible reconcile — d54×g13 dust_table G3/G4 keep-one (both ported 157/327 mid-flight)

d54 research/131 §8.1 row "G3/G4 — CLOSED ×2 — KEEP-ONE NOW OWED" and
§8.3.11 (which still *recommends* a shape) are **SUPERSEDED by execution**:

- **Both lanes ported the SAME arms mid-flight**: d54 `641561c2`
  (`DustDrawGlobals` + `get_alpha_with_globals`, 2 witnesses) vs g13
  `daf7b7804` (`get_alpha_mouse_text` + `get_alpha_disco`, 3 witnesses).
  g13's probe found the drops unclaimed at its read; d54's closure was
  in flight. Same verbatim maths (157: `/100−1.6`, +50, cap 255, uncapped
  alpha, f32 boundaries 190→a29 / 255→a94; 327: `!dayTime` gate + THREE
  R/G/B NextFloat draws + the `(int)(sum)*255` dead-raise quirk, sum<1).
- **The keep-one is EXECUTED on g13** (`8fd62bd7d`): d54-canonical wins;
  g13's split seams + their 2 witnesses RETIRED; the d54 blob taken
  VERBATIM plus g13-only additives (module note, `FloatLog` rig,
  `w4_g13_globals_seam_gate_census`, `w4_g13_draw_tint_census_pins`).
- **Blob arithmetic re-verified THIS round**: d54 tip `0426245f2`
  `dust_table.rs` = `58305d0a8` (779 ln, unchanged since `1d16ef4e`);
  g13 tip `eef0202cc` = `300c2647a` (874 ln at the seal base).
  `git diff 58305d0a8 300c2647a --stat` = **98 insertions, 0 deletions**
  (strict superset; deletion-line count 0 re-counted this round).
  **Merger instruction: take the g13 retire blob VERBATIM; d54's file
  needs no edit.** d50 is uncontested (`a0c5ce4d5` = merge-base, empty
  diff — its research/243→304 §4 receipts this third-party).
- **The exact drop-hunks** (what the merger must NOT resurrect from
  `daf7b7804`, 1-based in that blob — published by d50's retire doc §4,
  re-cited here): `get_alpha_mouse_text` **:314-334**, `get_alpha_disco`
  **:336-382**, `w4_g13_get_alpha_157_the_mouse_text_pulse` **:743-776**,
  `w4_g13_get_alpha_327_night_draws_three_and_never_raises` **:797-848**.
  The per-fn dedupe table stays research/210 §6 (was 70). Witness
  arithmetic: 5 gross → 3 net (d54's 2 + g13's census pair; the −2
  deduction stands, d54 §8.4).
- The DRAW-lane wiring stays hunk-published: research/210 §7 W1-W4
  against g67's live entries (cull+else-kill g67-owns `2c452edd`,
  ancestor-verified; d130 `16a792d56` carries the fold per d54 §8.1 R3).

## 3. The AddLight 94/94 census (fresh C# re-count + the SEAL witness)

Fresh `Select-String 'Lighting\.AddLight' decompiled/Terraria/Dust.cs`
at this seal: **94 sites exactly**. Split re-derived from the line list:
**42 FIRST-ladder** (`:485`…`:1046` — 258/309/325, the 86-92/286 gem
switch, 262/263-264/230/172/182/261/254/255/213-260/157/206/163/205/170/
156/234/174, the torch chain `:859-1020`, 269) in `dust_lights.rs`
(S1-E-5) + **52 SECOND-chain** (`:1061`…`:2312`) in `dust_lights_b.rs`
(g13 `a234107ac`). The full 52-line register is the module doc + the new
witness `w4_g13_seal_the_52_line_census_covers_every_addlight_site`
(WRITTEN NOT RUN): the (line, type) pair table, 42+52=94, one emitting
probe per site (51 emit; gates open), and the ONE cited drop pinned.

**The `:2275` accounting (d54 G5 CORRECTED, honored here):** the 226
customData-Color second light is **LIVE in C#** — the producer census
(d54 `c5a0e646`, re-read this round: `ParticleOrchestrator.cs:1171/1184/
1199` `SpawnLightningExplosionDust` sets `customData = dust.color` with NO
`noLightEmittance`; `Projectile.cs:39844` + `StormLightningParticle.cs:215`
both double-set `noLight = (noLightEmittance = true)` at `:39851`/`:222`
so THEY never fire it). The port drops it wholesale (the `Dust` record
has no customData, `particles.rs:100-102`) — **a cited gap, not an
uncounted site**. The merger reconstruction contract (d54 §7.2b, exact):
at EVERY Color producer `customData == dust.color` (CloneDust copies
both, `Dust.cs:280/:284`), so the feed reads `num83 * d.color.rgb / 255`
gated `!d.no_light_emittance` — no field needed. The Player-follow
halves (`:2265`/`:2239`/`:2323-2332`) stay genuinely dropped. Dark
sharers unchanged: 274/292 (15/57/58 arm), 134 (130-133), 223 (219-222).

**Post-wiring replay retire (the g13×d50 merge contract):** d50's retire
doc §6 re-pinned every `dust_lights_b` replay key at its post-wiring
`dust_physics.rs` (blob `60bc9112e`): 66/267 +0.02 (`:603`/`:615`),
20/21/231 +0.005 (`:622`), 41 +0.007 (`:854`), 44 +0.0085 (`:871`),
27/45 burn+push (`:628-654`), 35 ng +0.03 (`:777`), 35 wet incl. the
`alpha == 255` boundary RESOLVED by the chain's `next_max(2)` (`:810`),
the ramp (`:888+`), the 267 noLight lift. Key line numbers spot-verified
against the d50 HEAD blob this round. After the retire the collector is
a pure POST-record reader; the 23 colour-pin witnesses re-pin on
POST-update records; no RNG order changes (the replays drew nothing).

## 4. The d50 retire reads — the resume is EXECUTED and PUSHED

The g13 receipt (research/206 §8, `f9566e5b4`) predicted a mechanical
apply of §4-6+8 at the dead-clean `2e243a2d` tip. **It happened**: d50
origin advanced `2e243a2d2` → **`7474ce3d9`** (verified `git ls-remote`
this round; worktree HEAD == origin, status clean):

| Commit | What | g13 contract consumed |
| ------ | ---- | --------------------- |
| `ea749c5ae` | pre_move-35 strip + update_dust head wiring + snowDust union | research/206 §4 VERBATIM (the 35 arm gone, the unused param dropped, the 299-305 shrink/lift kept + wired at `:468-472`; g13's inline increment canonical, transplanted witness included); the G2 ONE-EDIT hunk (d54 `84eb739b`) applied |
| `807350de0` | type_chain into the live minecart tick + the dust_lights decrement retire | research/206 §5 arm-by-arm: 206 `d.scale*0.4`, 174 `cap(d.scale,0.6)`, 230 `(d.scale−0.02)*0.6`, 309 `d.scale/0.97`, 261 pre-drift recovery (fadeIn boundary documented) — all re-read at the d50 HEAD `dust_lights.rs` this round (`:408`/`:447`/`:320`/`:218`/`:352-355`) |
| `e84e6bd76` | the phase-1 arithmetic witness re-derivation (in-chain generic tails) | d50's own compile-window finding; g13 unaffected |
| `7474ce3d9` | RETIRE FINAL docs (research/243 — **loses 243 to g58, registry §7.4 → 304**) | the third-party keep-one receipt (§2 above) + the lights_b replay keys (§3 above) |

**The d54-bible rows flip at the merge:** "g13 flag-b pre_move 35 strip —
OWED" → EXECUTED; "g13 flag-c dust_lights decrement retire — OWED" →
EXECUTED (d50's half; the lights_b replay half rides the g13×d50 merge,
keys published); "G2 snowDust++ — OWED" → the hunk is APPLIED on d50, the
d130 tick-resolve landing (`:1069`/`:1126` shape) stays the merge's.
The `main_dust.rs` three-way union + moon-lord fuse are unchanged merge
mechanics (research/202 §5-6).

## 5. Research FINAL — the registry-number renames EXECUTED

Per registry-150 §3 (d54's collision registry, round-1 assignments;
§7.4 is the current authority and leaves these four untouched):

| Old | New | Winner keeping the old number | Refs re-pointed this seal |
| --- | --- | ----------------------------- | ------------------------- |
| `65-dust-main-sites-3.md` | `187-dust-main-sites-3.md` | f17 kinds-coverage (11-20T20:49) | ANALYSIS :39252, PORT_MAP :759, doc-202's addendum |
| `68-dust-main-residual-sweep.md` | `202-dust-main-residual-sweep.md` | f37 mood-reconcile (02:48) | ANALYSIS, PORT_MAP, ROADMAP, INDEX, doc-187, `main_dust.rs:24`, `dust_lights_b.rs`, `draw.rs:608`, anchors :900 |
| `69-dust-second-chain-lights.md` | `206-dust-second-chain-lights.md` | g46 liquid-reconcile (02:58) | ANALYSIS, PORT_MAP, ROADMAP, INDEX, docs 202/210, `dust_lights_b.rs`, anchors :900 |
| `70-dust-draw-tint-final.md` | `210-dust-draw-tint-final.md` | d53 proj-dust-site-batches (03:49) | ANALYSIS, PORT_MAP, ROADMAP, INDEX, docs 202/206, `dust_table.rs`, anchors :901 |

Content-preserving (`git mv` + header renumber + provenance note); every
`research/NN` §-citation on this branch re-pointed (grep-verified ZERO
old-number refs remain in docs/, crates/, anchors.txt). **Merger note:**
these four renames are EXECUTED — the merger must NOT re-apply the
registry §3 rows for g13; d54's OWN 65→186 / 66→194 renames still ride
the dust-union merge per the registry's d54 note. Foreign-lane citations
of the OLD numbers (e.g. d50's docs citing research/69) are THEIR
renames to do, not touched here.

## 6. SEAL verdict

**The W4-g13 dust-main family is SEALED**: the 18-commit ledger closes
with G1 94/94 (the fresh C# re-count + the census witness), Main COMPLETE
(zero unclaimed windows), the G3/G4 keep-one landed and third-party
receipted (the g13 blob wins, 98/0 superset re-verified), the d50 resume
EXECUTED + PUSHED (flag-b/flag-c consumed verbatim), the registry renames
EXECUTED, and every remainder a published mechanical contract: the
lights_b replay retire (§3 keys), the DRAW-lane W1-W4 (research/210 §7),
the three-way main_dust union (research/116 §6), the d130 tick landing
(d54 §7.2a piece 3). Nothing in this family needs porting again.

## Verification record (this seal)

- `git status --porcelain` clean at base `eef0202cc`; branch
  `task/W4-g13-dust-main-sites-3`; explicit `git add` only; no merge, no
  primary write, no other branch touched.
- Fresh C# reads (direct paths, ≤100-line windows): `Dust.cs` AddLight
  grep (94 lines listed), `:2129-2244`/`:2243-2337` (the second chain +
  the 226/331/272 arms), `:455-514`/`:1400-1454` (head/pre-move/43);
  `ParticleOrchestrator.cs:1158-1242` (the LIVE :2275 producers);
  `Projectile.cs:39830-39859` + `:68940-69024` (the gated producers);
  `StormLightningParticle.cs:90-134` + `:205-226` (the :222 double-set).
- Blob checks: `58305d0a8..300c2647a` = 98 insertions / 0 deletions
  (re-counted); d50 `dust_table.rs` == merge-base `a0c5ce4d5`; d50
  wiring line-keys re-read at `e84e6bd76` blobs (`dust_physics.rs`
  `:603/:615/:622/:777/:810/:854/:871`, `dust_lights.rs`
  `:218/:320/:352/:408/:447`).
- Registry: d54 tip `0426245f2` registry-150 re-read (§7.4: 308+ FREE);
  fresh 361-tip `ls-tree` sweep — zero ≥308 claimants; the 281/282 race
  observed and avoided (six 15:53-16:07 first-adds enumerated via
  `git log --all --diff-filter=A`).
- Renames: `git mv` ×4, headers + provenance notes, refs grep-verified
  clean. Witness `w4_g13_seal_the_52_line_census_covers_every_addlight_site`
  WRITTEN NOT RUN (code-only mandate; the compile window is the
  merge-verify wave's, d54 §8.3.13).
- No cargo/tests/rustfmt/graphify invoked.