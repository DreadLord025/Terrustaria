# 93 addendum — W4-f33 persist fold-input + the d19 949 drop-hunks (closeout)

**Status: FINAL — the fold-input LANDED on g88 (v11), the a6b re-compare
adjudicated (§3), the event final-verify done (§7); §8 is the FINAL
merger instruction.** This branch (`task/W4-f33-torch-god-favor`, retire
pass atop `7b334ea7`) keeps its own persist shape — it does NOT rebase to
v10/v11 (the canonical union writer is g88's; the merger folds per this
doc). Code-only lane: no cargo/tests/rustfmt/graphify; nothing here was run.

Inputs read at closeout probe (2026-11-21), UPDATED at the retire pass:

| Ref | Tip read | State |
| --- | --- | --- |
| `origin/task/W4-g88-life-fruit-use` | ~~`c8985b53`~~ → **`a9f1c29d`** (persist fold 7 FINAL — the **v11 TORCH FOLD LANDED**) | writer v11: f33's pair IS line 16 (103 §2b); the v7 reader gained the f33 lone-all-0/1-pair rule (`f33s_torch_only_v7_loads_positionally`) + 135 §2's six-line rule; §1's fold-input is CONSUMED — receipt below |
| `origin/task/W4-g89-aegis-fruit-use` | `59864256` (g89f) | research/122 carried on g88 (v9 addendum + §6/§7 fold receipts) |
| `origin/task/W4-g90-artisan-bread-use` | `b3b82d07` (g90c) | research/**135** published (the mechanical fold-input); g88 fold 6 already folded g90b `e76ee133` |
| `origin/task/W4-a6-crystals-powerups` | ~~`8ed96ee5`~~ → **`d7de8bd5`** (a6b — the POWERUP 4-way master reconcile) | research/**138 PUSHED** (`git ls-tree` verified on the new tip) + five fidelity fixes; its 5043 arm's achievement credit is now ROW-TARGETED `complete_condition` — the §3 keep-one re-compared below. NOTE: 138 §0 read f33 at `333479e0` ("completes NO achievement") — STALE: the f33 tip carries the full-body 5043 use arm + the named dispatcher (§3) |
| `origin/task/W4-d19-proj-ai-181-190` | `1054c396` (**d19b active** — the parked-intents follow-up over `a4185f89`) | the 184 slice + the dodge witness + the f33 event-contract paragraph landed; §2 below pins its exact drop-hunks |
| `origin/task/W4-c36-achievement-notify-save` | `585fa0a8` | the `GAIN_TORCH_GODS_FAVOR` row exists (blob-identical `achievements_data.rs`, §3 below); c36 carries its own 5043 use arm |
| this branch | `8a000407` | `.trplayer` writer **v7** biome-torch pair; `.plr` pair both halves + the legacy-279 shim |

research/135 (read) does NOT know f33's v7 line — its §2 recognition rule
counts g90b's six 1-token lines only. ~~research/138 (unread — unpushed)~~
**138 is PUSHED (a6b `d7de8bd5`) and read at the retire pass** — it
adjudicates the a6-vs-g90b shimmer use-arm keep-one AND claims the 5043
arm for a6 on the pre-`8a000407` f33 probe; §3 below re-compares against
the LANDED f33 arm and keeps the f33 full body. §1's mapping table was
f33's input to the g88 fold — **CONSUMED at `a9f1c29d` (receipt below)**.

### 1-receipt. The fold LANDED — 93a §1 ↔ 103 §2b (retire pass)

g88 `a9f1c29d` (persist fold 7 FINAL) implemented §1a's recommended v11
shape VERBATIM: the writer bumps v10 → **v11** appending
`unlockedBiomeTorches UsingBiomeTorches` as the LAST post-dye line AFTER
the loadout block (103 §2 line 16); token 7 stays DD2-RESERVED (the pair
is NOT an unlock token — §1a's rejection of the token options upheld);
the v7 reader gained the **f33 lone-all-0/1-pair rule** (§1's collision
resolution — "no maxima pair can be all-0/1, the permanent halves start
100/20", witness `f33s_torch_only_v7_loads_positionally`), which
supersedes §1's life-pair-first bounded-loss recommendation with a
LOSSLESS read (103 §4's matrix row: "LOSSLESS … NOT a life pair"). The
`Player.using_biome_torches()`/`set_using_biome_torches()` accessors ride
at f33's exact spot after `luck()`; the restore writes the setter. The
sidecar line's conditional DROP-recommendation is published at 103 §2b
rule 3 (gated on 132 §2c's character-load hydration + exit-save seams).
**This branch's `persist.rs` writer/reader sites now carry
DEPRECATED-BY-SUPERSET banner comments (commit `46ceb43d`, comments-only,
still compiling) pointing at this receipt.** f33's four witnesses
(§ "Witnesses on this branch") survive — g88 re-pinned the pair round-trip
as `w4_f33_the_biome_torch_pair_round_trips_the_sidecar` on its v11 writer.

---

## 1. MY `.trplayer` v7 line — the exact shape/position (fold source)

`crates/tr-app/src/persist.rs` on this branch. Fixed prefix identical to
103 §2 lines 1–7 (version / pos+hp / 50 inv pairs / coins / armor /
kills / dye). The post-dye block is **ONE line, TWO 0/1 tokens**:

| File line | Content | Writer site (`persist.rs`) | `SavedPlayer` fields (decl) | C# `.plr` anchor |
| --- | --- | --- | --- | --- |
| 8 | `unlockedBiomeTorches UsingBiomeTorches` | `:83-87` (the `format!("{} {}\n", …)` tail of `save`, header `"7\n"` at `:60`) | `unlocked_biome_torches` (`:34`) / `using_biome_torches` (`:37`) | `Player.cs:54777`/`:55241` + `:54778`/`:55242` (release ≥ 229; ctor `:2046` / the `builderAccStatus[11]` property `:3950-3963`) |

Token 1 = the PERMANENT unlock (`Player.unlockedBiomeTorches`). Token 2 =
the builder-toggle half (`Player.UsingBiomeTorches` — the getter is
`unlockedBiomeTorches && builderAccStatus[11] == 0`; the save writes the
PROPERTY, `persist.rs:86` reads `player.using_biome_torches()`). The pair
rides the vanilla `.plr` write order (`:54777` then `:54778` — the first
two bools of the 229+ unlock block, BEFORE bread `:54779` and the
crystal/fruit family `:54781-54786`).

### Reader gates (the exact code the fold replaces)

- Version arm `persist.rs:95-103`: `"3" | "4" | "5" | "6" | "7" => false`
  (no migrate); `"1" | "2" => true`; anything else (incl. `"8"`/`"9"`/
  `"10"`) → `None`. **This reader rejects every g88/g89 superset file** —
  by design, the superset reader wins at the merge.
- Armor gate `:127`, kills gate `:135`, dye gate `:150`: `"7"` in each
  `matches!` — same shape as g88's.
- v7 block `:159-167`: line 8 is REQUIRED (`lines.next()?` at `:160`);
  token-count tolerant — `parts.next().is_some_and(|v| v == "1")` per
  half, so a 1-token v7 line (g89's aegis / g90's bread) loads the torch
  pair false without error (witness
  `w4_f33_a_one_token_v7_line_loads_the_torch_pair_false`, `:281`).
- App restore `app/mod.rs:2948-2949`: `p.unlocked_biome_torches =
  saved.unlocked_biome_torches; p.set_using_biome_torches(saved.using_biome_torches)`
  — the toggle half restores through the SETTER (`builderAccStatus[11] =
  !value ? 1 : 0`, `player.rs:10600-10602`), so a saved OFF toggle keeps
  its OFF state. Both restores sit BEFORE the hp clamp (the flags never
  feed hp — g88's pairs-before-clamp rule, 103 §5, is unaffected).

### THE COLLISION 103/135's matrices do not know

f33's v7 line is a **2-token** post-dye line. Under the g88 superset's
v7 walk (103 §3, `persist.rs` g88 tip `:409-427`): the `2 =>` arm maps
the FIRST two-token line to the LIFE pair — f33's `1 1` torch line reads
as `statLifeMax = 1, statLifeMax2 = 1` (the permanent half clamps
`.min(500)` → 1). The unlock stays false AND the maxima corrupt to 1.
Under g89's v9 reader the same line reads as a life pair too (122 §2's
corruption class). 135 §2's six-1-token-line recognition rule does not
cover this shape (it is ONE 2-token line, not six 1-token lines).

**The recognition problem is REAL and bounded:** f33's torch-only v7
(`1 1` alone) is token-shape-identical to g88's pre-reconciliation
life-only v7 (`f1eb9b60`, `statLifeMax statLifeMax2` alone). No pushed
reader can self-identify a lone 2-token v7 line. The fold contract:

1. **Post-fold files are unambiguous** — the superset writer emits the
   torch pair at its own reserved position (§1a below), never as a bare
   2-token line.
2. **Pre-fold f33 files** (a lone `1 1` / `0 0` / `1 0` / `0 1` 2-token
   v7 line) cannot be distinguished from a life-only v7 by shape. The
   merger's choice, recorded for the keeper: read a lone 2-token v7
   line as the LIFE pair (103 §3's existing walk — the g88 life-only
   predecessor is the older, more widespread shape) and accept that a
   pre-fold f33-only file loses its torch unlock on the superset reader
   (the character re-uses the favor — the use arm is once-ever per
   character, so the loss is bounded to the unlock flag; the `.plr`
   pair, §1b, carries the permanent truth across the merge). The
   alternative (torch-first) corrupts every g88 life-only file's maxima
   — strictly worse. **Recommendation: life-pair-first, document the
   bounded f33-pre-fold loss like 103 §3's bread→aegis ambiguity.**
3. A 2-token v7 line FOLLOWED by other post-dye lines is the partial
   superset shape (103 §4's `717e0640` 3-line row) — the walk already
   handles it; f33 never wrote one.

### 1a. THE MAPPING TABLE — my pair vs the union's coordinates

Three coordinate systems meet: my v7 line, the `.plr` bool positions,
and 103 §2a's unlock-line tokens. **The torch pair is NOT an unlock-line
token** — the vanilla writes it at `:54777-54778`, OUTSIDE the
`:54780-54787` unlock family the line-8 token map covers (103 §2a's
table starts at `usedAegisFruit` `:54782`). It is also NOT a world flag
(the 5336/5343 boundary, 103 §6 rule 4) — it is per-PLAYER.

| Field | my v7 token | `.plr` write / read | 103 §2a token | UNION position (recommended) |
| --- | --- | --- | --- | --- |
| `unlockedBiomeTorches` | line 8 token 1 | `:54777` / `:55241` (gate 229) | — (outside the unlock family) | a NEW 2-token post-dye line, appended AFTER the loadout block per 103 §6 rule 5 |
| `UsingBiomeTorches` | line 8 token 2 | `:54778` / `:55242` (gate 229) | — | same line, token 2 (the pair order = the vanilla write order) |

**The recommended union shape (v11):** the torch pair rides its OWN
2-token line appended AFTER the g55 loadout block (103 §6 rule 5 — a
new post-dye line, never a reorder of lines 8–15). The line
self-identifies by POSITION (the first line after the third 41-token
set line) — no token-count sentinel needed, the loadout block's shape
walk already consumes everything ahead of it. The writer bumps v10 →
**v11** (the DD2 token-7 reservation, 103 §2a, is untouched — the torch
pair is not token 7). The v10 reader back-reads: a v10 file has no line
after the sets → the torch pair loads false/false (the correct default
for a character that never used the favor). The v7 walk keeps 103 §3
unchanged plus the bounded lone-2-token ambiguity above.

The alternative (a 7th/8th unlock-line token pair) is REJECTED: it
would renumber 103 §2a's reserved token 7 (DD2) and mix a non-unlock
family pair into the fruit-first line — 103 §6 rule 4's append-never-
renumber discipline forbids it.

**Does a6 carry the same pair? NO.** a6's tip (`8ed96ee5`) writes
`w.boolean(false)` at BOTH `.plr` slots (`player_file.rs:867-868` —
hardcoded false, the unlock never persists), has NO `.trplayer` writer
(its `persist.rs` is the v6 base — no post-dye block at all), and its
use arm sets `builder_acc_status[11] = 0` directly (`powerups.rs:353`)
with no `using_biome_torches()` property (the getter/setter pair is
f33's, `player.rs:10594-10602`). g88's tip likewise writes both `.plr`
bools false (`player_file.rs:867-868`) and carries no torch fields in
`persist.rs`. c36's tip: same false/false `.plr` write, no sidecar
torch fields. **f33 is the ONLY pushed writer of the pair** — the
fold-input is unclaimed, no keep-one needed on the persist half.

### 1b. MY `.plr` halves (independent of the sidecar, 103 §6 rule 3)

`player_file.rs` on this branch: the `PlayerFile`/`PlayerFileWrite`
structs carry `unlocked_biome_torches` + `using_biome_torches`
(`:126`/`:130`, `:845`/`:848`); the release-326 writer emits them at
`:897-898` (`:54777`/`:54778`); the reader takes them at `:338-342`
(`:55241-55242`, the 229+ gate, BEFORE the 256 bread bool and the 260+
six-unlock walk — the walk stays aligned, witnessed); the legacy-279
subset shim reads them as unlock_bools[1]/[2] (`:752-753`). The
merge dedupe: a6/g88/c36's `w.boolean(false)` lines at `:867-868`
replace with f33's `w.boolean(file.unlocked_biome_torches)` /
`w.boolean(file.using_biome_torches)`; their readers' discarded
`r.boolean()` at the bt1/bt2 slots (`:329` on both tips) take f33's
named reads. The golden fixture stays false/false (byte parity).
Witness: `w4_f33_the_biome_torch_pair_round_trips_the_plr` (`:1259`).

### Witnesses on this branch (WRITTEN, NOT RUN)

`persist.rs:231` the pair round-trip · `:254` v6 loads no torch unlock ·
`:281` the 1-token v7 tolerance · `player_file.rs:1259` the `.plr` pair
round-trip. The merger keeps all four; the sidecar pair re-pins to the
v11 writer's line position at the fold.

---

## 2. The d19 949 keep-one — the EXACT drop-hunks (d19b tip `1054c396`)

The keep-one verdict is research/93 (`fff0bd29`): **KEEP f33's
`projectile_ai_torch_god.rs`, DROP d19's 184 slice.** d19b landed the
parked intents (`c5a7e5d3`) + the docs follow-up (`1054c396`) AFTER the
93 note's `a4185f89` probe — the 184 section itself is unchanged, but
the dodge witness is NEW and the line numbers shifted. The exact hunks
on the d19b tip, for the merger:

### 2a. `crates/tr-app/src/projectile_ai_sword_styles.rs`

| Hunk | Lines (tip `1054c396`) | Content | Action |
| --- | --- | --- | --- |
| A | `392-481` | the `// aiStyle 184` banner (`392-394`) + `torch_god_self_hurt` (`396-434`) + `ai_bad_torch_luck` (`436-481`) | DELETE (f33's `ai_184_bad_torch_luck` + `self_hurt_players` replace; the C# bodies are the same window `:42787-42822`/`:15025-15062`) |
| B | `1457-1482` | the witness `w4_d19_the_torch_fireball_drains_and_rearms_collision` (doc `1457-1459`, `#[test]` `1460`, fn `1461-1482`) | DELETE — the witness is WRONG (asserts `tile_collide` on the drain tick itself, `:1481`; the C# if/else arms it the NEXT tick — f33's `w4_f33_the_ai_budget_drains_by_the_speed_then_arms_tile_collide` pins the correct shape) |
| C | `1484-1525` | the NEW d19b witness `w4_d19_the_torch_fireball_self_hurt_is_dodgeable` (doc `1484-1494`, fn `1496-1525`) | DELETE the d19 copy; the CONTRACT survives — f33's `hurt_with_source(…, true, 949)` rides the same `hurt_inner` dodge lane (`player.rs:4356-4374`, sash→belt→brain→shadow). The f33 closeout adds the equivalent witness on the f33 arm (§5 below) |
| D | `17-19` | the module-header census bullet `- aiStyle 184: …` | REPOINT to `projectile_ai_torch_god.rs` (the canary rule — never delete a census line outright) |
| E | `50-52` | the RNG-contract bullet `- 184: four dusts per tick …` | REPOINT — the f33 census is 32 draws/tick (28 rainbow), not 4; cite `projectile_ai_torch_god.rs`'s `new_dust_census` |
| F | `72-74` | the movement paragraph's `184 starts tileCollide = false …` sentence | KEEP the fact, repoint the module reference |
| G | `107-111` | the deviations bullet `* SelfHurtPlayers (184): …` | MOVE to the f33 module's doc (the folds are identical — no port fields for sweater/gravestone/trapDebuffSource) or repoint |
| H | `1309-1312` | the row witness's `let torch = proj_data(949) …` block inside `w4_d19_the_rows_carry_their_styles_and_slices` | KEEP (the row survives the dedupe — the witness reads the merged row; the `ignore_water` assertion is d19's comment half, merge it onto the survivor row) |

### 2b. `crates/tr-app/src/projectile.rs`

| Hunk | Lines (tip) | Content | Action |
| --- | --- | --- | --- |
| I | `17398-17410` | the `// aiStyle 184` comment (`17398-17401`) + the `184 => { … ai_bad_torch_luck … kill_step(world); }` arm (`17402-17410`) | DELETE — f33's arm (`projectile.rs:17554-17572` on the f33 tip) replaces it; both sit between the same 187/185 neighbours, keep ONE. f33's arm guards `kill_step` on `active` (equivalent for 184 — the AI never kills) |
| J | the death hook | d19 has NO 949 Kill wiring | ADD f33's `kill_census` hook (f33 `projectile.rs:20015-20029` — the `kind == TORCH_GOD && !died_out_of_world` gate, the Item10 cue + the 160-draw census) |

### 2c. `crates/tr-app/src/projectile/data.rs`

| Hunk | Lines (tip) | Content | Action |
| --- | --- | --- | --- |
| K | `1137-1139` | the `TORCH_GOD` const + doc | DROP one copy (f33's `data.rs:79-81` survives — merge d19's `ProjectileID.cs:2452` citation onto it) |
| L | `3684-3687` | the 949 row + comment | DROP one copy — the tuples are IDENTICAL (`(4.0, 4.0, 184, -1, TIME_LEFT_DEFAULT, 0, false, false, false, RESET_STATE)`); f33's `data.rs:3675-3682` survives with BOTH comment halves (f33's friendly/hostile-lane citation + d19's ignoreWater-exclusion citation) |

### 2d. Docs / anchors (the canary rule)

- `anchors.txt:899` (the d19 sword-styles row) names
  `AI_184_BadTorchLuck 949 (:42787-42822 + SelfHurtPlayers :15025-15062)`
  in its feature text — REPOINT that clause to
  `projectile_ai_torch_god.rs`, never delete the row (its marker
  `pub(crate) fn ai_nights_edge` survives the dedupe).
- `docs/ANALYSIS.md:39401-39419` (the d19b "184 BadTorchLuck torch-god
  event contract (f33)" paragraph) says **"The f33 branch is NOT on
  origin"** — STALE at closeout: `task/W4-f33-torch-god-favor` IS pushed
  (`8a000407`, verified `git ls-remote`). The paragraph's spawn contract
  (`spawn_with_data_ai_owned(949, owner, x, y, vx, vy, dmg, 1.0, dust_id,
  distance)`) is exactly what f33's `torch_god.rs:472` drain calls — the
  merger updates the paragraph to the landed state, the contract itself
  needs no change.
- The d19 PORT_MAP row (`docs/PORT_MAP.md:758`, the spore-children row)
  cites "the 184 torch-god event contract documented (f33's branch
  absent from origin…)" — same stale clause, same update.

Everything else in d19 (182/183/185/186/190 + the 12 other rows + the
92 cloud family + the 1126 spore drain) is untouched by this dedupe.

---

## 3. The torch-god ACHIEVEMENT chain — f33 use → c36 row (verified)

**The c36 `complete_condition` row EXISTS.** `achievements_data.rs` is
blob-identical on both tips (`e048692067a92e920d5b4dd70b20d8a1bc7ca98e`,
`git rev-parse HEAD:… origin/task/W4-c36…:…`): the
`GAIN_TORCH_GODS_FAVOR` row at `:1004-1010` —
`AchievementInitializer.cs:360-362`, `Category::None` (the registry's
one uncategorised row), icon 109, one condition `Cond::Flag("Use")`
(`CustomFlagCondition.Create("Use")`).

**The chain, end to end:**

1. f33 use arm `torch_god_favor_use.rs:117` —
   `app.achievements.handle_special_event(24)` (`Player.cs:45233`).
2. f33 named dispatcher `achievements.rs:482-487` (the `24 =>` arm at
   `:484`) — `self.complete_condition("GAIN_TORCH_GODS_FAVOR", "Use")`
   (`AchievementsHelper.cs:367-369`'s
   `GetCondition("GAIN_TORCH_GODS_FAVOR", "Use").Complete()`).
3. `complete_condition` (`achievements.rs:446` on f33, `:448` on c36 —
   the same named-row seam) completes EXACTLY that row's "Use"
   condition; the row's single condition closes → the row completes →
   `save()` + the completion event (`AchievementManager.cs:180-187`).
4. The row's data (step 0) is c36's blob — the chain terminates on a
   row that exists on the c36 tip. **Verified: no missing row, no
   broadcast over-completion.**

**The keep-one adjudication (three pushed 5043 use arms):**

| Lane | Arm | Seam | Achievement wire | Verdict |
| --- | --- | --- | --- | --- |
| f33 `8a000407` | `torch_god_favor_use.rs::torch_gods_favor_item_check` | the `:43600` permanent-item seam (`editing.rs:2007`), the recall press-edge machine | `handle_special_event(24)` → the NAMED dispatcher | **KEEP** — the full C# body: ApplyItemTime BEFORE the grant, the UsingBiomeTorches setter, the net-aware bestiary credit (host/client split + the msg-51 request), the generic consumable tail, the `.plr`/sidecar permanence |
| a6 `8ed96ee5` (`6e37aaac`) | `powerups.rs:344-359::use_torch_gods_favor` | the `powerup_item_check` dispatch (`powerups.rs:101`) | ~~`notify_custom("Use")` — the BROADCAST~~ **SUPERSEDED at a6b `d7de8bd5`**: row-targeted `complete_condition("GAIN_TORCH_GODS_FAVOR","Use")` (`powerups.rs:360-362`, 138 §3 fix 1) | DROP — re-compared at the retire pass (§3b): the broadcast is fixed but the arm is still not the full C# body (no setter accessor, no net split, no msg-51, SP-only credit, ApplyItemTime moved to the dispatch tail) |
| c36 `585fa0a8` | `editing.rs:16332-16354::use_torch_gods_favor` | the summon-gate `is_summon` list (`editing.rs:2017`, the 5043 entry) + `:2092` | `complete_condition("GAIN_TORCH_GODS_FAVOR", "Use")` — named, correct | DROP — the arm is a reduced copy (no UsingBiomeTorches write, no net split, the stack pay inline); c36's enter-world re-check (`achievement_hooks.rs:173-176`, the `OnPlayerEnteredWorld` `:192-195` arm) is a SEPARATE wire and SURVIVES the dedupe — it re-completes the row for a character whose `.plr` carries the unlock (idempotent, `complete_condition` on a completed row is a no-op) |

The merger drops a6's + c36's 5043 arms (and c36's `is_summon` 5043
entry at `editing.rs:2017` + the `:2088-2094` call block), keeps f33's
arm + the named dispatcher + c36's enter-world re-check. The dispatcher
itself: f33's `handle_special_event` (the event-id switch) and c36's
direct `complete_condition` calls coexist — c36's other arms (the
crystal/fruit `use_permanent_powerup`, the equip checks) keep calling
`complete_condition` directly; f33's switch is the `HandleSpecialEvent`
shape for the event-id sites. No conflict: both land on the same
`complete_condition` seam.

Witnesses (f33, WRITTEN NOT RUN):
`achievements.rs:839` the row is a Use flag not a progression ·
`:868` event 24 completes ONLY the torch row (the other eight "Use"
rows stay open, the event queues once, the second 24 is a no-op) ·
`torch_god_favor_use.rs` the use grants the achievement + the bestiary
credit with the named-row isolation.

### 3b. The a6b re-compare (retire pass — 138 read, `d7de8bd5` vs the f33 tip)

138 §1's 5043 row keeps **a6's** use arm ("the faithful setter …
supersedes c36") and records "f33 owns the EVENT machine + flag READS,
completes NO achievement". **That f33 claim is STALE**: 138 §0's probe
read f33 at `333479e0` (the event-machine commit, BEFORE the use arm
landed). The pushed f33 tip (`8a000407` + closeout) carries the FULL
`ItemCheck_UseTorchGodsFavor` body in `torch_god_favor_use.rs` AND the
`HandleSpecialEvent(24)` named dispatcher. Re-comparison, C# re-read
`Player.cs:45226-45243` (whole body, retire pass):

| C# element | f33 (`torch_god_favor_use.rs`, tip) | a6b (`powerups.rs:347-366`, `d7de8bd5`) | verdict |
| --- | --- | --- | --- |
| Gate `:45228` (`itemAnimation > 0 && !unlocked && ItemTimeIsZero`) | `run_favor_arm`'s head verbatim + the press-edge machine + the CCed/dead refusals (`:42598-42603`) | `run_powerup_arm`'s head — the same two legs; the start gates ride a6's shared machine | both faithful (a6b §3's "not a gap" note holds) |
| `ApplyItemTime(sItem)` `:45230` — BEFORE the grant | in-arm, first write (`torch_god_favor_use.rs:110`) | moved to the dispatch tail AFTER the arm returns true (`powerups.rs:115`) — observation-equal in SP, order deviates | **f33 faithful** (C# order) |
| `unlockedBiomeTorches = true` `:45231` | `p.unlocked_biome_torches = true` | same | equal |
| `UsingBiomeTorches = true` `:45232` (the property setter `:3960-3963`) | `p.set_using_biome_torches(true)` — the PROPERTY accessor pair f33 declared (`player.rs:10594-10602`), the getter is what the v11 writer reads (103 §2b rule 2) | `p.builder_acc_status[11] = 0` — the setter's BODY inlined, no property | **f33 faithful** (the property is the C# surface; 138's "faithful setter" claim describes the same write — both land `builderAccStatus[11] = 0`, but only f33 carries the accessor the superset writer/restore need) |
| `HandleSpecialEvent(this, 24)` `:45233` | `handle_special_event(24)` — the event-id switch (`AchievementsHelper.cs:288-379` shape) → `complete_condition("GAIN_TORCH_GODS_FAVOR","Use")` | `complete_condition("GAIN_TORCH_GODS_FAVOR","Use")` direct (138 §3 fix 1 — the broadcast RETIRED) | **both faithful NOW** — one row, one condition; f33 keeps the dispatcher seam for future event ids, a6b calls the shared helper directly. Either dispatch survives; keep ONE helper |
| Bestiary credit `:45234-45239` (`netMode == 0` leg) | host-or-SP register + `try_grant_bestiary_100_net` + the host `98` broadcast; a CLIENT skips the local register | SP-only register ("the port is SP"), no net split, no 100% hook | **f33 faithful** (the net-aware seam + `NPCKillsTracker.cs:46-49`'s hook) |
| `SendData(4)` `:45240` | the identity re-push fold documented (the flags ride the next sync — recorded fold) | absent (folded silently, doc line only) | f33 documented |
| `SendData(51, …, 5f)` `:45241` | `send_torch_god_bestiary_request()` — the CLIENT's msg-51 request (`MessageBuffer.cs:2514-2520`) | **absent** | **f33 only** |
| Consumable tail `:43280-43326` | in-arm, with the `:43323` 0-stack `itemTime = itemAnimation` re-arm | a6's `consume_one` airs out but has no `:43323` re-arm | **f33 faithful** |
| Persistence | the `.plr` pair both halves + the v7 sidecar line (deprecated-by-superset, §1-receipt) | none (138 §1 agrees: "NO sidecar. No `.wld`"; the `.plr` pair rides f44) | f33 owns the halves (dedup onto f44c's identical fields, 103 §2b rule 4) |

**VERDICT (final): KEEP the f33 full-body arm
(`torch_god_favor_use.rs` + its `editing.rs` seam + the named
dispatcher), DROP a6b's 5043 arm** (`powerups.rs:347-366` + the `5043 =>`
dispatch entry `:104` + the `5043` leg of `is_powerup` `:71` + a6b's two
torch witnesses `:1071`/`:1093` — re-pin their assertions onto f33's six).
The original broadcast objection is RETIRED (a6b fixed it — credit where
due), but the surviving gaps (ApplyItemTime order, the property accessor,
the net split, the msg-51 request, the `:43323` re-arm) all sit on the
C# body f33 ports whole. **Both dispatchers are faithful now → keep one,
either — pick the f33 full body** (its `handle_special_event(24)` switch
lands on the SAME `complete_condition` seam a6b calls directly; the
helper dedups a6b == c36 == f33, one copy). a6b keeps its machine for the
REST of the powerup cluster (138 §8 rule 5's exceptions already list
29/109/1291/5326; 5043 JOINS that exception list — this doc is the
adjudication input g88's 103 §5a and a6b's 138 §1 both deferred).
Everything else in 138 (the four other fidelity fixes, the persist-layer
split, the `.wld` byte-compare, the v18 65-byte union) is UNAFFECTED —
f33 endorses them; its five-fix consumer shapes match g90's, which 103
§5a already keeps.

---

## 4. The minigame music-101 priority census (no other track overrides)

The chain: `Player.torch_god.happy_fun_torch_time` →
`scene_scan.rs:465` (`SceneMetrics.AddPlayerEffects`,
`SceneMetrics.cs:770-773`) → `tr-core/scene.rs:162`
`in_torch_god_minigame` → `app/frame/world.rs:195` `MusicEnv` →
`audio.rs:2696-2699` the ladder arm (`Main.cs:12542-12545`).

**The census — every `new_music` writer in `audio.rs`, in execution
order, vs the minigame flag:**

| Site | Track | Runs during the minigame? | Verdict |
| --- | --- | --- | --- |
| `:2586`/`:2658` the `musicVolume == 0` mute | 0 | yes — the user's volume slider | NOT a track override (the C# mute, `Main.cs:12189`); faithful |
| `:2676-2690` the `gameMenu` title ladder | 51/60/86/70/8/11 | only on the front end — the minigame cannot run there (no world) | exclusive arm, returns first; faithful (`Main.cs:12455`) |
| **`:2696-2699` the minigame arm** | **101** | **the flag IS the gate** | **the track; returns — every arm below is unreachable while it holds** |
| `:2707` the boss ladder | 93/5/… | NO — the minigame arm returned | outranked (the C# order: credits 89 → minigame 101 → bosses; witnessed) |
| `:2713` eclipse 27, `:2718` shimmer 91, `:2733` town 46/47, `:2738` sandstorm 40, `:2743` underworld 36, `:2748` space 42/15, `:2753` temple 26, `:2758` dungeon 23, `:2764` mushroom 29, `:2771-2780` evil 33/16/10/8, `:2785` meteor 2, `:2789` graveyard 53, `:2795` desert 21, `:2802-2810` jungle 54/52/7/55, `:2816` snow 20/14, `:2822` hallow-UG 11, `:2844` UG 4/31, `:2851-2857` hallow-surface 52/19/44/9, `:2862` storm 2/52, `:2869-2873` ocean 2/46/47/22/43, `:2880-2892` night/rain tail, `:2901-2903` day 1/18/44 | all | NO — all sit BELOW the minigame arm in `decide_new_music`, each behind its own `return` | none can override 101 |
| `:2910-2917` `apply_music_box_override` | the box's track / 0 | YES — runs AFTER `decide_new_music` returns (`update`, `:2591`) | **the ONE override above the minigame** — a placed music box (or the silence box) beats EVERY world branch including the minigame, exactly like the C# (`Main.cs:11076-11350` sits behind `DecideOnNewMusic`); faithful, not a deviation |
| the ambience pair 28/45 (`:2631-2646`) | rain/wind loops | yes — but they ride `update_common_track` on their OWN channels, never write `new_music` | not music-track overrides (the C# ambience tracks, `Main.cs:11527-11600`) |

**No other `music.update` call site feeds the world env**: the only
other caller (`app/mod.rs:1517`) is the front-end tick with
`menu: true` + `..Default::default()` (the minigame flag false). The
credits-roll 89 arm (`Main.cs:12536`, above the minigame in the C#) is
the unported `CanPlayCreditsRoll` event — folds off, documented in the
`MusicEnv` field doc (`audio.rs:2419-2422`). The TOW ladder's track-87
arm (`Main.cs:11863`) is the unported getGoodWorld/swapMusic path —
outside `decide_new_music`, cannot fire.

**Verdict: during the minigame, track 101 holds against every world
branch; the only overrides are the C#-faithful ones (the volume mute,
the front-end ladder, the music box).** Witness:
`audio.rs:3404` `w4_f33_the_torch_god_minigame_track_beats_the_boss_ladder`
(101 alone, 101 over a live boss 93, the boss resumes after the event,
the menu ladder still wins) + the closeout's full-ladder sweep (§5).

---

## 5. Closeout witnesses added on this branch (WRITTEN NOT RUN)

- `audio.rs::w4_f33_the_minigame_track_holds_against_every_world_branch`
  — the §4 census as a witness: the flag set + every below-ladder env
  input armed at once (boss, eclipse, shimmer, town, sandstorm,
  underworld, space, temple, dungeon, mushroom, evil, meteor,
  graveyard, desert, jungle, snow, hallow, storm, ocean, blood moon,
  rain, windy) → still 101; the music box override still wins (the
  C#-faithful exception); the flag cleared → the ladder resumes.
- `projectile_ai_torch_god.rs::w4_f33_the_self_hurt_is_dodgeable` — the
  d19b dodge contract (§2 hunk C) re-pinned on the f33 arm: a
  belt-armed owner rolls the 1/10 BEFORE damage, the dodge swallows the
  hit; without the belt the same overlap hurts. The f33 arm carries the
  ByProjectile source through the same lane (`hurt_inner`'s dodge block,
  `player.rs:4356-4374`).

## 6. Cross-refs (updated at the retire pass)

- research/93 (`93-torch-god-949-keep-one.md`) — the arm-by-arm verdict
  this addendum's §2 executes; its `a4185f89` line numbers are
  superseded by §2's `1054c396` hunks. Its §3 keep-one table is
  re-adjudicated by §3b above (a6b row-targeted now; the f33 full body
  still wins on the rest of the C# body).
- research/103 (g88 tip ~~`c8985b53`~~ → **`a9f1c29d`, v11 FINAL**) —
  THE merger contract; §1a's v11 torch-pair line LANDED (§1-receipt);
  §9 is its FULL merger persist-checklist.
- research/122 (g89d, carried on g88) — the v9/v10 unlock-line
  contract; the torch pair is OUTSIDE its token map (§1a).
- research/135 (g90 tip `b3b82d07`) — the g90 fold-input; its §2
  six-line rule LANDED in the g88 v11 reader (`g90bs_six_line_v7_loads_positionally`,
  the bounded-misread witness retired per `a9f1c29d`).
- research/138 (a6b `d7de8bd5`) — PUSHED + read; its 5043 keep-one is
  re-adjudicated by §3b (its f33 probe `333479e0` predates the use arm);
  its four OTHER fidelity fixes + the persist-layer split + the v18
  65-byte union stand endorsed.

---

## 7. The torch-god EVENT final-verify (retire pass — the whole window re-read)

`Player.cs:17813-17910` (`TorchAttack`) re-read WHOLE against
`crates/tr-app/src/torch_god.rs` (tip), plus the surrounding arms
`:17718-17811` / `:18060-18088`. **Verdict: NO C# arm missed — every leg
of the window is ported; the base stands.** Line-by-line:

| C# leg | Lines | Port site | Status |
| --- | --- | --- | --- |
| `whoAmI != Main.myPlayer` head | `:17815-17818` | folded (single local player — the `local_owner` gate rides the 949 AI instead) | documented fold |
| Surface abort → `EndOngoingTorchGodEvent` | `:17819-17823` | `torch_attack_step`'s head (`torch_god.rs:225-228`) — feet-Y `(pos_y as f64) < world_surface * 16.0`, BEFORE the buff/cadence | faithful |
| `AddBuff(80, 2)` Blackout every event tick | `:17824` | `buffs.add_buff(80, 2)` after the abort gate | faithful |
| 21-tick volley cadence (`++` then `<= 20` return, reset 0) | `:17825-17830` | `torch_god.rs:234-238`; witness `…volley_cadence_is_21_ticks` | faithful |
| ±100-tile box, `Utils.Clamp(…, 10, max−10)` ×4 | `:17833-17840` | `utils_clamp` (max-test-FIRST fold) + the four bounds | faithful |
| Candidate scan x-OUTER/y-inner, `active & type==4 & frameX<66`, 400-cap with BOTH breaks | `:17841-17861` | `'outer:` loop + `TORCH_ATTACK_CANDIDATES = 400` (`:57129` `new int[400]` re-verified); witness `…caps_at_400_and_orders_x_outer` | faithful |
| End arm: `num == 0 \|\| attacks >= maxTorchAttacks(200)` → Relight + flag clear | `:17862-17865` | `torch_god.rs:267-270` (`MAX_TORCH_ATTACKS = 200`, `:57128` re-verified) | faithful |
| `netMode == 1` SendData(4) on the end arm | `:17866-17869` | net-only fold; the flag itself rides `4 PlayerInfo` bitsByte16 bit 1 (`NetMessage.cs:183` — netglue `:1132`/`:1145`/`:1917` send, `:1376` receive) | covered by the identity sync |
| Survival `>= 95` → `RequestNewItem(5043, ReserveForLocalPlayer)` | `:17870-17873` | `favor_drop` + `push_favor_drop` — the NewItem velocity pair `Next(-30,31)*0.1` / `Next(-40,-15)*0.1` drawn IN-TICK (`Item.cs:49749-49750` re-read whole `:49726-49751`: `Prefix(0)` no draw, 5043 not ItemNoGravity → the else pair), the 18×18 box (`Item.cs:40440-40441` re-verified), SP reserve no-op | faithful |
| ONE `Main.rand.Next(num)` pick | `:17881` | `rng.next_range(0, num)` — byte-equal stream witness | faithful |
| Re-validate picked cell | `:17884` | `torch_god.rs:295-300` | faithful |
| Damage `(int)(40f * HostileProjectileDamageMultiplier.Sample)` | `:17887` | `torch_attack_damage` — the identity-line curve, four difficulties witnessed | faithful |
| `clamp(frameY/22, 0, Count−1)` → `TorchID.Dust[…]` | `:17888-17889` | `TORCH_ID_COUNT = 24` + the verbatim `TORCH_DUST` table | faithful |
| Snuff `frameX += 66` + the unlit-list record + counter++ | `:17890-17893` | in-tick list/counter writes, the tile snuff defers via `pending.snuff` (drain applies `+= 66` with the same lit-torch gate) | faithful (the pending convention) |
| `SendTileSquare` | `:17894` | net-only fold (the port's tile-sync lane owns squares) | documented fold |
| Spawn `(x*16+8, y*16)`, aim = Length→Normalize→×8 | `:17895-17899` | `launch_aim` — XNA f32-sum/f64-sqrt Length + the reciprocal-multiply Normalize (zero-vector NaN kept, no guard — C# has none) | faithful |
| `NewProjectile(Misc(10), …, 949, dmg, 1f, owner, dust, dist)` + `ai[0]/ai[1]` re-writes + `netUpdate = true` | `:17900-17903` | `spawn_with_data_ai_owned` + `friendly = false` (the stock `Projectile.cs:788` the pool default overrode) + `net_update = true` | faithful |
| Finale `(num == 1 && attacks >= 95) \|\| attacks >= 200` → `torchFunTimer = −180` | `:17904-17907` | `torch_god.rs:331-335`; the 201-tick-gap witness | faithful |
| Surrounding: `EndOngoingTorchGodEvent` (`:17718-17729`), the event head + cooldown decrement (`:17733-17742`), `RelightTorches` (`:17797-17810`, cooldown FIRST then the `frameX >= 66` −66 loop), `KillMe`'s abort (`:38855-38858` → `player.rs:3709`), the trigger tail (`:18066-18087`: the six gates + the 59-slot 5043 inventory scan + `nearbyTorches = 0` AFTER) | — | `end_ongoing_torch_god_event` / `try_recalculating_torch_luck_head` / `apply_relight` / `start_death` / `torch_god_trigger_check` + `world_sim/player.rs:602`'s scan (the mouse-item join is the recorded drop convention) | faithful |

The volley's only RNG is the one `Next(num)` + the grant's velocity pair
— both witnessed byte-equal; the dust censuses ride the 949 AI module
(the `UpdateProjectiles` stream, §2/research/93). **Nothing to add,
nothing to fix.** (Witnesses WRITTEN NOT RUN — code-only lane.)

---

## 8. FINAL merger instruction (research/93 + 93a — the retire closeout)

The keeper executes, in order:

1. **949 projectile (research/93 + §2):** KEEP f33's
   `projectile_ai_torch_god.rs` + dispatch arm + Kill-census hook + lane
   exclusions. DROP d19b's hunks A/B/C/I/K/L exactly as §2 pins
   (`1054c396`); repoint D/E/F/G/H + the anchors/ANALYSIS/PORT_MAP stale
   clauses (§2d). Merge both comment halves onto the surviving row.
2. **Event machine (§7):** f33's `torch_god.rs` + the `luck.rs` seam +
   the `world_sim` drain land AS-IS — final-verified whole-window, no
   other lane ports any leg (138 §8 rule 5 agrees: "Keep f33's Torch God
   event machine + flag reads").
3. **5043 use arm (§3/§3b):** KEEP f33's full-body
   `torch_god_favor_use.rs` + the `editing.rs:2007` seam + the
   `handle_special_event` dispatcher. DROP a6b's `powerups.rs:347-366`
   + its `5043` dispatch/`is_powerup` legs + its two torch witnesses
   (re-pin onto f33's six); DROP c36's reduced arm + its `is_summon`
   5043 entry/call block (§3's table); KEEP c36's enter-world re-check
   (`achievement_hooks.rs:173-176`) and the GAIN_TORCH_GODS_FAVOR row
   (blob-identical). Dedup `complete_condition` (a6b == c36 == f33 —
   ONE helper; a6b's clears `frac`, keep that line on the survivor if
   the shared `Condition` carries `frac`).
4. **Persist (§1/§1-receipt):** the g88 **v11** superset
   (`a9f1c29d`) is THE `.trplayer` writer/reader — f33's v7 sites are
   marked DEPRECATED-BY-SUPERSET (`46ceb43d`, comments-only). Follow
   103 §9's full checklist (L1 f44c base + named-bool dedups / L2 the
   v11 superset / L3 the v18 65-byte union per 138 §5). The sidecar
   torch line drops ONLY when 103 §2b rule 3's two seams land (132 §2c);
   until then it stays the dual-carrier. f33's `.plr` halves dedup onto
   f44c's identical named fields; the four f33 witnesses re-pin per
   §1-receipt.
5. **Music (§4):** the 101 ladder arm + the box override stand verified
   — no merger action.

**Everything on this branch is either the keeper or the fold-input;
nothing here needs a follow-up pass. The branch retires at this commit.**