# 316 — W4-f39 v18-claim INPUT for g71c (the closeout receipt, code-only)

Lane: `task/W4-f39-weather-slimerain`, closeout tip `da058b714` (this doc's
commit follows it). Written 2026-11-21 for the **g71c union-final merger**
(`origin/task/W4-g71-bg-layers-2`, read at tip `8224c94f1` — the
OVERRULE-CORRECTION phase). This is an INPUT doc: it re-receipts f39's
`.trwld` v18 claim at the CURRENT tip and hands the merger the exact
truncation-site moves. It does NOT re-open the layout: **research/92
§1-FULL (seating 103 §L3-ARBITER A3, `d09d7e10a`) stays THE byte-order
authority**; f44's research/132 stays the L1/L2 + cross-layer merger-bible.

Number: **316 — RENUMBERED 244 → 281 → 316** (registry 150 §9 UNIFIED
FINAL, d54 tip `b243c7086`, FROZEN §8.1 row f39: 281 = f26's §7.3
assignment (68-7h), so f39's executed 244→281 self-rename `fec65992f` was
a CONTESTED self-pick into the assigned 281–303 block → re-issued 316;
chain 244→281→316 SUPERSEDES the unexecuted §7.3 244→299 row — **299 is
FREE again**). The first leg (244 → 281, at the f39 seal) was
registry-150 first-pushed-wins, d54 round-2 refresh `72226e500`:
the original 244 pick
(a 358-tip scan at closeout 5, max seated 243) raced g48's correction
phase: **g48 `244-c28-18-send-switch.md` first-add `2b61530b4`
2026-11-21T14:30:16+02:00 WINS 244**; this doc's first-add `8704691a3`
14:32:01 is 1m45s later → LOSER. Both pushes postdate the registry's
final sweep, so the §6.3 cutoff rule applies: losers self-adjudicate and
take ≥281 after a fresh sweep (the 244-280 block is fully assigned).
A fresh **386-tip / 22 694-row** `ls-tree docs/research/` sweep at the seal
finds 247-300 empty at every tip (246 = d10's executed `247` assignment
off-by-one, `d89380ca1` 14:52:30 — d10's own post-cutoff chain, not this
lane's to fix); **281 was the first free number ≥281 at that sweep —
SUPERSEDED: 281 was f26's reserved assignment, the round-4 §9 adjudication
re-issues this doc to 316**. Content-preserving
rename per registry §3: body unchanged, header + every ref re-pointed
(ANALYSIS / ROADMAP / INDEX on this branch). f39's earlier phases
published NO numbered research doc (the census rows live in
ANALYSIS/PORT_MAP), so this stays the lane's first number claim. The
seal receipt + the slimerain-family bible: research/320 (chain 282→320,
the same §9 adjudication).

## 1. The claim — registry row #8 re-receipted at the closeout tip

92 §1-FULL.1 row #8 reads: **f39 moon bosses DUP**, tip `bd5a5a24`,
"byte-identical five flags (inherited c39 code)", 5 B dup of row #7 (c39,
claimant of record per research/114 §0.2). That verdict STANDS at the
closeout tip — the five bytes did not move:

* **Writer** `crates/tr-core/src/save.rs:438-447` (after the v17 fishron
  byte at `:437`): the C# save-row order — `downed_halloween_king`,
  `downed_halloween_tree`, `downed_christmas_ice_queen`,
  `downed_christmas_santank`, `downed_christmas_tree`
  (`WorldFile.cs:1347-1351`), each `u8::from(bool)`.
* **Reader** `save.rs:830-838`, the `if version >= 18` arm — the exact
  inverse, same order. A pre-v18 file loads all five down (the vanilla
  reset arm).
* **`VERSION = 18`** at `save.rs:75`.
* **Union seat**: [51..56] in 92 §1-FULL.2's A3 table (crimson [50] →
  moon [51..56] → underworld [56] → pets [57..60] → empress [60] →
  books [61..63] → identity [63..107]). f39's block contributes **ZERO
  extra bytes** — the keep-one adjudication (#8 = #7) drops f39's
  writer/reader hunks wholesale at merge; c39's identical block seats.
* **Fixed head stays 107; total tail `107 + width*height`.** f39 claims
  no other `.trwld` block (the SlimeSky/music/emote closeout items add
  no persistence — the sky is runtime-only, the music env is derived,
  the emote gates read the already-claimed five flags).

**The `.wld` side (separate stream, NOT the union):** f39 Phase B owns
the C# flags-block bytes — `wld/sections/flags.rs:147-151` (the five
downeds, `WorldFile.cs:1347-1351`, tail offsets 1893-1897 per
`wld/mod.rs:2082-2086`) + `flags.rs:209-210` (the `forceHalloweenForToday`/
`forceXMasForToday` pair, `:1386-1387`, offsets 2002-2003 per
`wld/mod.rs:2101-2102`). 92 §1-SEAL.2's compose table already defers
force-today to f39 ("f45-owned (force-today defers to f39)") — that
deference is now DISCHARGED: the writes/reads are live on this tip
(`flags.rs:599-600` reads). The five downed writes are World-owned
(`NPC.cs:79210-79238` death arms, `world_sim/npcs.rs`); c39's research/114
§0.2 keeps the `.trwld` claim, the `.wld` bytes compose field-ownership
only (f45 base + f39's live hunks, no offset moves).

## 2. The truncation-site moves — f39's column at the closeout tip

92 §1-FULL.4's `c39 / f39` column was read at `bd5a5a24`; the closeout
commits (`746bd0037`..`da058b714`) did NOT touch `save.rs`, so the sites
are unchanged — re-pinned here at `da058b714` for the merger's arithmetic
(every `+5`/`-5` becomes `+107`/`-107`; the dense block is d109's, not
f39's):

| Site | f39 line (this tip) | Current shape | Union move |
| --- | --- | --- | --- |
| v12 `let tail = …` | `save.rs:1310` | `… + 9 + 1 + 5 + 1 + 5` (the last `+5` = v18) | last `+5` → `+107` |
| `EMPTY_V16_TAIL` | `save.rs:1339` | `EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 5` | → `EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 107` |
| v14 `truncate(len-1-5-1-…)` | `save.rs:1528` | `- 1 - 5 - 1 - 5` | last `-5` → `-107` |
| v18/v17 round-trip truncation | `save.rs:1553` | `v17.truncate(v17.len() - 5)` | `-5` → `-107` |
| v5/v6/v7 `V6_FLAGS_LEN+…` | `save.rs:1198` | `… + 1 + 5` | last `+5` → `+107` |
| v13 `2+V6_FLAGS_LEN+…` | `save.rs:1588` | `… + 1 + 5 + 1 + 5` | last `+5` → `+107` |
| mid-record cuts (TE/item/banner/bestiary/NPC/tiers) | `save.rs:1608/:1724/:1844/:1959/:2086/:2187` | `-2`/`-3` only — **v18-UNAWARE** (unarmed at this tip, same as the `bd5a5a24` read) | **re-arm all six** `-2-107`/`-3-107` per record shape (g72's `2b02345c` pattern, 92 §1-FULL.4's merge instruction) |

The round-trip witness `w4_f39_the_seasonal_moon_boss_downeds_round_trip_in_v18`
(`save.rs:1535`) keeps its assertions; only the truncation length moves.
The pre-v18 back-compat witness (`save.rs:1550-1556`) patches the version
to 17 after the peel — unchanged by the union (the whole tail unread).

## 3. What the closeout added (no persist interaction)

For the merger's diff-scope check (`git diff origin/main..branch --stat`
must list only task files), the closeout's four commits on top of
`bd5a5a24a`:

| Commit | Item | Files | Persist impact |
| --- | --- | --- | --- |
| `746bd0037` | SlimeSky (`GameContent/Skies/SlimeSky.cs` port, `slime_sky.rs` + the sandwich band through `render/sky.rs`/`bg.rs`, `Sky_Slime_1..4` atlas 8119→8139 px fits 8192) | 10 files, +814 | NONE — runtime sky, private world-seed-pinned stream |
| `c786ffb32` | KingSlime spawn-from-rain (`NPC.SpawnOnPlayer(plr,50)` faithful scan, `spawner.rs`; the kill-150 threshold arm rewired, `world_sim/npcs.rs`) | 3 files, +344/−14 | NONE — the counter flip stays unconditional (`NPC.cs:79093`) |
| `cb9955257` | the event music tracks (`flag24`→48 at `Main.cs:12654` + the moon tail 30/32 at `:12982-12989`, `audio.rs` wrapper + `MusicEnv` inputs) | 2 files, +196/−2 | NONE — derived env |
| `da058b714` | the seasonal emote/help-text gates (`emotes.rs`: `seasonal_boss_emote_gates` reading the five flags + `seasonal_help_text_gate`, `Main.cs:39752/:39757`) | 1 file, +131 | NONE — reads the already-claimed flags |

**Co-lane hand-off (W4-g54):** this branch's emote wheel is the flat
`EMOTE_COUNT` roster; the gated wheel is g54's `EmoteGates`, whose
`from_world` pins the five seasonal flags `false` ("unowned"). At the
merge g54 reads `w.downed_halloween_tree`/`_king`,
`w.downed_christmas_tree`/`_ice_queen`/`_santank` instead (field names
identical). `emotes.rs::seasonal_boss_emote_gates` is the 1:1 reference
(`UIEmotesMenu.cs:250-269`); no second wheel was ported here.

All witnesses WRITTEN NOT RUN (code-only lane: no cargo/rustc/tests/
rustfmt/graphify in any closeout phase). Verified by re-reading the C#
in ≤100-line windows and hand-arithmetic; the atlas budget measured with
the node harness (`tools/atlas_sim.js`), not cargo.

## 4. Merger checklist (f39's obligations, discharged or handed off)

1. `.trwld` v18: DROP f39's writer/reader hunks (keep-one, c39 seats the
   identical five at [51..56]); move the seven truncation-site families
   per §2; re-arm the six mid-record cuts.
2. `.wld`: f45 base + f39's live five-downed + force-today hunks
   (`flags.rs:147-151`/`:209-210`, reads `:506-510`/`:599-600`) —
   field-ownership compose, no offset moves (92 §1-SEAL.2).
3. g54 merge: re-point `EmoteGates::from_world`'s five `false` pins at
   the World fields (§3 hand-off).
4. The resolved `World.xmas`/`halloween` stay runtime-only (re-derive at
   dawn/load, Phase B) — NOT persisted, no union seat.
