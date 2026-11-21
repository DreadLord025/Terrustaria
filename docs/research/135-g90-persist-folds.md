# 135 — W4-g90 persist fold-input: the exact token-lines, offsets and reader-gates

**Status: FINAL — CONSUMED by g88 fold 7 (`a9f1c29d8`, writer v11).**
The fold-input below (§1–§6) stands as published; §8 records the
consumption receipts and the two SUPERSESSIONS the pushed writers
force: the unlock-token order (§3's recommendation overridden by g88's
written crystal=4/arcane=5/pearl=6, per 103 §8 + 138 §6) and the `.trwld`
union size (§4's 21 bytes overridden by 138 §5a's FIVE-way **68-byte**
table — g87's pet trio at 19-21 moves this branch's pair to **22/23**).
§9 records the 138 keep-one receipts (this lane's carrier-keep list).
This branch (`task/W4-g90-artisan-bread-use`) keeps its own persist
shape — it does NOT rebase to v10/v11 (the canonical union writer is
g88's; the merger folds per this doc + its §8/§9 corrections).
Code-only lane: no cargo/tests/rustfmt/graphify; nothing here was run.

Inputs read at probe (2026-11-21):

| Ref | Tip read | Persist state |
| --- | --- | --- |
| `origin/task/W4-g88-life-fruit-use` | `2ceec691` (fixup over `62c255e0d` fold 4) | writer **v8** superset, tolerant v7/v8 reader; `docs/research/103-persist-v7-layout.md` is THE merger contract; g88d v9-fold in flight |
| `origin/task/W4-g89-aegis-fruit-use` | `e9ec8dcfc` (g89e over `4344bd50` g89d / `78766306` g89c) | writer **v9**, one 3-token unlock line; `docs/research/122-persist-v9-addendum.md`; `.trwld` VERSION stays **17** (no v18 tail) |
| `origin/task/W4-g71-bg-layers-2` | `e2d085ae4` (research/92) | writer **v18**, 3-byte surface tail; 92 §1 publishes the g70⨉g71 **19-byte UNION** contract (g71c extending) |
| this branch | `73367e0a` | `.trplayer` writer **v7** six-line block; `.trwld` **v18** two-byte tail |

---

## 1. MY `.trplayer` v7 block — the exact lines (fold source)

`crates/tr-app/src/persist.rs` on this branch. Fixed prefix identical to
103 §2 lines 1–7 (version / pos+hp / 50 inv pairs / coins / armor /
kills / dye). The post-dye block is **six lines, ONE 0/1 token each**:

| File line | Content | Writer site (`persist.rs`) | `SavedPlayer` field (decl) | C# `.plr` anchor |
| --- | --- | --- | --- | --- |
| 8 | `ateArtisanBread` | `:103` | `ate_artisan_bread` (`:35`) | `Player.cs:54779` / `:55245` (gate 256; ctor `:56077`) |
| 9 | `usedAegisCrystal` | `:104-105` (flag loop `:104-112`) | `used_aegis_crystal` (`:39`) | `:54781` / `:55255` (ctor `:56078`) |
| 10 | `usedArcaneCrystal` | `:106` | `used_arcane_crystal` (`:42`) | `:54783` / `:55257` (ctor `:56080`) |
| 11 | `usedGalaxyPearl` | `:107` | `used_galaxy_pearl` (`:45`) | `:54784` / `:55258` (ctor `:56081`) |
| 12 | `usedGummyWorm` | `:108` | `used_gummy_worm` (`:48`) | `:54785` / `:55259` (ctor `:56082`) |
| 13 | `usedAmbrosia` | `:109` | `used_ambrosia` (`:51`) | `:54786` / `:55260` (ctor `:56083`) |

The order is the vanilla `.plr` write order `:54781-54786` **minus the
g89-owned fruit bool** (`:54782`), with bread (`:54779`) leading. The
5338 Aegis Fruit flag has NO line on this branch — g89's lane.

### Reader gates (the exact code the fold replaces)

- Version arm `persist.rs:125`: `"3" | "4" | "5" | "6" | "7" => false`
  (no migrate); `"1" | "2" => true`; anything else (incl. `"8"`/`"9"`)
  → `None`. **This reader rejects every g88/g89 file** — by design, the
  superset reader wins at the merge.
- Armor gate `:152`, kills gate `:160`, dye gate `:175`: `"7"` added to
  each `matches!` — same shape as g88's.
- v7 block `:193-207`: line 8 is REQUIRED (`lines.next()?` at `:194`,
  `trim() == "1"` → bread); lines 9–13 are OPTIONAL (`if let Some(line)
  = lines.next()` per flag, `:196-206`) — absent → false. So the
  branch's own first-pass bread-only v7 (`1f1b88f0`) loads all five
  unlocks false (witness `w4_g90_a_bread_only_v7_loads_without_the_unlocks`,
  `:332`).
- App restore `app/mod.rs:2944-2952`: bread then the five flags, AFTER
  the hp clamp (`:2939`) — flags never feed hp, so the g88
  pairs-before-clamp rule (103 §5) is unaffected; at the merge the five
  restores join g88's aegis/gummy restore block (gummy dedups — g88
  fold 3 already restores `used_gummy_worm`).

### Witnesses on this branch (WRITTEN, NOT RUN)

`persist.rs:274` bread round-trip · `:289` v6 loads no bread · `:310`
five-flag round-trip (`w4_g90_the_shimmer_unlocks_round_trip`) · `:332`
bread-only v7 back-compat. The merger keeps all four; `:310`/`:332`
re-pin to the superset writer's version at the fold.

---

## 2. THE COLLISION the fold must fix (not in 103's matrix)

103 §4's back-compat matrix knows only my FIRST pass (`1f1b88f0`,
bread-only v7, one 1-token line). The lane-7 extension (`e76ee133`)
writes SIX 1-token lines and **no pushed reader loads it correctly**:

- **g88 superset v7 walk** (103 §3): first 1-token line → aegis (my
  BREAD lands in the fruit slot); second 1-token line → bread (my
  CRYSTAL lands in the bread slot); every further 1-token line
  OVERWRITES the bread slot again (`persist.rs` g88 `:336-343` — the
  `1 =>` arm has no seen-cap). Net effect on my file: aegis=bread,
  bread=ambrosia (the LAST line wins), crystal/arcane/pearl/gummy lost.
- **g89 v9 reader** (122 §1): version `"7"` reads line 8 token 1 as
  aegis (my bread again), tokens 2/3 absent → false; lines 9–13 never
  read. Same corruption class 122 §2 documented for the bread-only
  file, now six lines deep.

### The mechanical recognition rule for g88d's legacy v7 arm

A v7 post-dye region of **exactly six lines, ALL 1-token** is
unambiguously THIS branch's lane-7 shape — no other pushed writer
produces it (g89 flag-only = 1 line; g90 first-pass = 1 line; g88
life-only = one 2-token line; 3-line superset = 1+2+2; 4-line =
1+2+2+1; g55 loadout-only = 2+41+41+41). Fold recipe:

```
if version == "7" && post.len() == 6 && post.iter().all(|t| t.len() == 1) {
    // g90 lane-7 block (135 §1): positional, .plr write order
    bread   = post[0][0] == "1";   // line 8
    crystal = post[1][0] == "1";   // line 9  -> unlock token 6
    arcane  = post[2][0] == "1";   // line 10 -> unlock token 5
    pearl   = post[3][0] == "1";   // line 11 -> unlock token 4
    gummy   = post[4][0] == "1";   // line 12 -> unlock token 2
    ambrosia= post[5][0] == "1";   // line 13 -> unlock token 3
    // pairs None, loadouts empty, aegis_fruit false (never written here)
}
```

Everything else keeps 103 §3's walk unchanged. The lone-1-token-line
ambiguity (bread-only vs aegis-only) stays as 103 §3 documents it —
this rule does not touch it. Add a back-compat row to 103 §4:
`g90 lane-7 six-line v7 | 7 | yes | all six flags read positionally`.

---

## 3. THE EXACT MAPPING TABLE — my lines/bools vs the union's unlock tokens

> **SUPERSEDED for rows 4/5/6 (§8.1):** g88 fold 6 (`c8985b534`) WROTE
> tokens 4/5/6 as crystal/arcane/pearl (the vanilla `.plr` order). The
> "UNION token (recommended)" column below (pearl=4/arcane=5/crystal=6)
> is the pre-fold recommendation only — the written order is binding
> (103 §8, 138 §6). Rows 1/2/3/7 and the bread line agree verbatim.

Three coordinate systems meet on the union's line-8 unlock line:
my v7 lines (§1), my `.plr` 260+ bool positions, and the reserved
token positions of 103 §2a / 122 §3.

| Field | my v7 line | my `.plr` bool # (260+ block) | `.plr` write / read | 103 §2a token | 122 §3 token | UNION token (recommended) |
| --- | --- | --- | --- | --- | --- | --- |
| `usedAegisFruit` (g89) | — (absent) | 2 | `:54782` / `:55256` | 1 LIVE | 1 LIVE | **1** |
| `usedGummyWorm` | 12 | 5 | `:54785` / `:55259` | 2 LIVE | 2 LIVE | **2** |
| `usedAmbrosia` | 13 | 6 | `:54786` / `:55260` | 3 RESERVED | 3 LIVE (v9) | **3** |
| `usedGalaxyPearl` | 11 | 4 | `:54784` / `:55258` | 4 RESERVED | 6 (append order) | **4** |
| `usedArcaneCrystal` | 10 | 3 | `:54783` / `:55257` | 5 RESERVED | 5 (append order) | **5** |
| `usedAegisCrystal` | 9 | 1 | `:54781` / `:55255` | 6 RESERVED | 4 (append order) | **6** |
| `downedDD2EventAnyDifficulty` | — | — (gate 182) | `:54787` / `:55265` | 7 RESERVED | — | **7** |
| `ateArtisanBread` | 8 | — (gate 256) | `:54779` / `:55245` | own line (103 line 11) | own line (122 line 11) | **own 1-token line after the mana pair** |

**The 103-vs-122 token-order conflict, resolved mechanically:** 103 §2a
reserves pearl=4 / arcane=5 / crystal=6; 122 §3 (written when 103 did
not exist in any ref) proposed appending crystal=4 / arcane=5 / pearl=6
"in vanilla .plr order". Both agree tokens 1–3 = fruit/gummy/ambrosia.
**No pushed writer has ever emitted a token 4+** (g89's v9 line stops at
3; g88's v8 line stops at 2), so NEITHER order carries a back-compat
obligation — the choice is free. Recommend **103 §2a wins** (it is the
published superset contract, its reader already skips tokens 3+, and
122 itself defers: "record the deviation here when a6 lands" — the
crystals are g90's now, not a6's; a6's claims were consumed-dropped per
g89b/g89c). g88d activating tokens 4/5/6 must follow 103 §2a's rule:
read the position in `load`'s v8/v9 arm, emit in `save`'s unlock line,
restore in `app/mod.rs` beside aegis/gummy, round-trip witness, update
the 103 table row RESERVED → LIVE (g90).

**Short-line back-reads after the fold** (all keep working):
v7 1-token (g89) / v8 2-token (g89b) / v9 3-token (g89c) lines read
tokens 1–3 and default 4–6 false — the g90 flags absent, exactly the
fresh-ctor state. My six-line v7 loads via §2's rule. The `.plr`
(`player_file.rs`) is INDEPENDENT of all this (103 §6 rule 3): this
branch already reads bools 1/3/4/5/6 (`player_file.rs:368-389`, the
fruit bool 2 consumed-dropped at `:377` for g89) and writes them
(`:989-996`, fruit slot `:992` stays false for g89); the legacy-279
subset keeps indices 3/4/6/7/8/9 (`:819-824`). The g89 merger flips
`:377`/`:992` to the field — one-line dedups, no gate moves.

### Use-path / consumer overlaps with g89 (merger dedup list)

- `use_shimmer_permanent_item`: g89b/c's core takes `(app, held, kind,
  already, set)` and owns the 5341/5342 arms; this branch's
  `editing.rs:16073` covers 5337/5339/5340/5341/5342 on the i37
  press-edge dispatch (`:2386`). Keep ONE core (g89's signature is the
  pushed precedent), extend it with the three crystal/pearl arms, keep
  this branch's dispatch seam. The 5341/5342 arms are semantically
  identical (same C# `:45411-45422`) — dedup, don't stack.
- `use_combat_book`: g89e `(app, held)` vs this branch `(app)`
  (`editing.rs:16003`); both arms 4382/5336, same flags/keys/folds.
  Keep one; this branch's is wired at `:2374`.
- `Player.used_ambrosia` + the booster triple: declared on BOTH
  branches at the same anchor (g89c and `player.rs:2160`/`:6904-6908`)
  — identical rows, clean dedup. `used_gummy_worm` pre-existed (i35).
- `.wld` book-two byte: g89e and this branch both write
  `flags.rs`'s `combatBookVolumeTwoWasUsed` + tail 2036 — same field,
  same gate 259; keep one (this branch's pair also owns the satchel
  byte g89e leaves `false`).

---

## 4. MY `.trwld` v18 tail — byte positions for the g71c union extension

> **SUPERSEDED offsets (§8.2):** the 21-byte union below was the
> g70⨉g71⨉g90 three-way contract. Two more v18 writers surfaced after
> publication — f45's 44-byte identity block and g87's 3-byte pet trio
> (`dfbcaf94b`) — and a6's research/138 §5a (`115ce655a`) adjudicates the
> FIVE-way **68-byte** union: g87's pets take offsets 19-21 (vanilla
> write order `:1392-1394` precedes the books `:1407-1408`), so THIS
> branch's pair MOVES to offsets **22/23** and f45's identity block
> rides 24-67. Every truncation constant below moves to **+68**, not
> +21. The byte semantics/gates are unchanged.

`crates/tr-core/src/save.rs` on this branch: `VERSION = 18` (`:72`).
Writer `:442-445` appends **2 bytes** directly after the v17
`downedFishron` byte (`:434`); reader `:832-835` (`if version >= 18`):

| Offset in MY v18 block | Byte | Field | C# `.wld` anchor |
| --- | --- | --- | --- |
| 0 | u8 0/1 | `World.combat_book_volume_two_was_used` | `WorldFile.cs:1407` write / `:2404` read gate 259 (`NPC.cs:6341`) |
| 1 | u8 0/1 | `World.peddlers_satchel_was_used` | `:1408` / `:2405` gate 260 (`NPC.cs:6343`) |

Pre-v18 loads both false (fresh-world default). Book ONE
(`combat_book_was_used`) has NO `.trwld` byte — its carrier is the
`.wld` flags half (i35, tail 1939, `wld/mod.rs:2083`); the recorded
remainder stands (`save.rs:438-441`).

### The union extension (input to g71c / the g70⨉g71⨉g90 merge)

Research/92 §1 fixes the g70⨉g71 union at **19 bytes**, forest-first.
This branch's pair appends AFTER it — the C# field order agrees
(the book/satchel bytes `:1407-1408` follow the setBG/treeBG bytes
`:1319-1379` in `WorldFileSave`), so the 92 §1 "least-surprising
order" rationale extends verbatim:

```
… v17 downedFishron byte (1)
v18 UNION tail (21 bytes), offsets from the start of the v18 block:
  [0..16]  g70 forest-set block   (tree_x 3×i32 LE + forest_bg_styles 4×u8)  — 92 §1
  [16]     snow_bg_style    u8    ┐ g71 surface bytes                        — 92 §1
  [17]     desert_bg_style  u8    │
  [18]     ocean_bg_style   u8    ┘
  [19]     combat_book_volume_two_was_used  u8   ← THIS BRANCH (gate-259 field)
  [20]     peddlers_satchel_was_used        u8   ← THIS BRANCH (gate-260 field)
```

Writer (after the 92 §1 union writes):

```rust
w.write_all(&[
    u8::from(world.combat_book_volume_two_was_used),
    u8::from(world.peddlers_satchel_was_used),
])?;
```

Reader (inside the same `if version >= 18` arm, after the 19 union reads):

```rust
world.combat_book_volume_two_was_used = r.u8()? != 0;
world.peddlers_satchel_was_used = r.u8()? != 0;
```

`VERSION` stays **18** (all three lanes bumped 17→18; 92's
forward-only rule applies — no released v18 exists, the union is the
first to land). g89's `.trwld` is still v17 (`e9ec8dcfc` `save.rs`), so
g89 contributes NOTHING to this tail — its book-two work is `.wld`-side
only (dedup per §3).

**Truncation-constant moves on THIS branch at the union merge** (the
92 §1 list extended; every site currently counts my `+2`, becomes
`+21`):

| Site (`save.rs`) | Current | Union |
| --- | --- | --- |
| `:1194` v6 downgrade truncate | `… + 1 + 2` | `… + 1 + 21` (the `+1` is v17 fishron) |
| `:1306` v12 `let tail = …` | `… + 5 + 1 + 2` | `… + 5 + 1 + 21` |
| `:1335` `EMPTY_V16_TAIL` | `EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 2` | `… + 1 + 21` |
| `:1524` v14 queen-slime probe | `len - 1 - 5 - 1 - 2` | `len - 1 - 5 - 1 - 21` |
| `:1547` pre-v18 probe in `w4_g90_the_book_two_and_satchel_flags_round_trip_in_v18` (`:1535`) | `len - 2` | `len - 21` |
| `:1576` v5 downgrade truncate | `… + 4 + 2` | `… + 4 + 21` |

The version-gate fixture (`dw_1_4_version_above_4_rejected`, `:1106`)
keeps its 18→19 repin; the writer-emits-18 pins (`:1081`, `:1663`)
stay. g70's/g71's own round-trip witnesses move 16/3 → 21 per 92 §1's
instruction, extended by this pair.

**`.wld` side (independent of the union, already landed here):** write
`flags.rs:234-235`, read gates `:628-633` (259/260), tail pins
`wld/mod.rs:2071-2072` (2036 = princess@2035+1, 2037 = +1), gate
witness `gate_v3_i35_the_combat_book_and_red_slime_flags_follow_their_version_gates`
(`:3644`, extended to v258/259/260). Book-one tail 1939 (`:2083`) and
its 204 gate untouched.

---

## 5. Combat-book CONSUMER census — flag landed, who is armed?

The brief's question: book 4382's damage buff (`NPC.combatBookWasUsed`,
`NPC.cs:6339`) — did the flag land, and is the consumer armed?

**Flag: LANDED on every carrier.** Use path `use_combat_book`
(`editing.rs:16003-16032`, dispatch `:2374`, once-ever WORLD gate,
45-tick cadence, `Misc.*Used` world text, consumable pay + TurnToAir);
`.wld` write/read (i35 book one; g90 book two, §4); `.trwld` v18 pair
(book two + satchel); net WorldInfo bitsByte9[0] + bitsByte11[6/7]
(`netglue.rs:2038-2040` client install / `:2132-2134` server push).

**Consumers, one by one:**

| Consumer | C# anchor | Port site | Armed live? |
| --- | --- | --- | --- |
| Town-NPC stat ladder, book ONE row (+250 lifeMax, ×0.8 knockback, +0.25 dmg, +8 def) | `NPC.cs:53018-53024` | `npc_ai_town.rs::town_stat_bonuses` (`env.combat_book`) | **YES — g90 feeder `8521139d`** (see below) |
| Town-NPC ladder, book TWO row (same four adds, stacks) | `NPC.cs:53025-53031` | same fn (`env.combat_book_two`) | **YES — same feeder**; witness `w4_g90_the_env_flags_feed_both_books_and_the_boss_ladder` + the live `update_npcs` witness `w4_g90_the_town_tick_feeds_both_combat_books_into_the_ladder` (750/750 HP, def 31) |
| Full-heal on growth (`(lifeMax > defLifeMax) & flag3`) | `:53032-53035` | the non-short-circuit `&` kept | **YES — same feeder** |
| Fishing `DidNotUseCombatBook` condition (book ONE only) | `GameContentFishDropPopulator.cs:196` (`Add(Legendary, 2, 4382, BloodMoon, DidNotUseCombatBook)` — a DROP condition, NOT a fishing-power penalty; the `Projectile.cs:21056` `Next(3)` inline arm is dead code, `FishingCheck_RollItemDropOld` is never called) | `fishing.rs:1450` via `BobberInputs.combat_book_was_used` (`projectile_bobber.rs:128`), fed live at `world_sim/projectiles.rs:327`; the 1/2 rule `fishing.rs:1691` | **YES** (census witness `w4_g90_the_combat_book_drop_dies_once_the_book_is_used`, `fdffccc4`) |
| TRAINING_DAY progression-40 (both books up) | `Player.cs:25695-25698` | `world_sim/player.rs:1625-1627` (`notify_progression_net(40)`, idempotent) | **YES** |
| Travel-shop +1 slot (SATCHEL, not the books) | `Chest.cs:1303-1306` | `shop.rs:808-810` (plain +1, no RNG draw), live via `world_sim/weather.rs:559` | **YES** |

**The town ladder IS armed now (g90 feeder, `8521139d`).** At the first
probe the `npc_ai_town.rs` module was UNWIRED (the b3/S1-D-3 lane's
code-only port; the live dispatch routed town kinds through
`npc.rs::update_guide`). The b3 lane has since pushed phase 1
(`96a5d88f0`, `ai_007_town_family` — but its env folds
`combat_book_two: false` and its `TownOutputs` drain is phase 2). The
g90 feeder wires the STAT half live on this branch:
`TownEnvFlags::from_world` (the b3 env's scalar half, BOTH book flags +
the boss progression off the live `World`) + a per-town-NPC
`town_stat_bonuses` pass in `world_sim/npcs.rs::update_npcs` ahead of the
slot loop (RNG-free, reset-then-read base, idempotent). The AI dispatch
stays b3's; at the merge b3's arm's ladder call dedupes onto the feeder
(same inputs → same writes, zero draws). The `(num2, num3)` attack
scales have no consumer until b3 drains `TownOutputs` — the defense/HP
writes are the live observable half.

---

## 6. Merge checklist (g88d / the merger, in order)

> **Status at the FINAL probe (§8):** steps 1's v7-rule + token
> activation are DONE on g88 (`a9f1c29d8`, v11 — the written token
> order per §8.1, NOT 103 §2a's old reservation); step 3's union is
> the 68-byte five-way table per §8.2/138 §5a, NOT 21. The checklist
> below stands as the original fold-input.

1. Keep g88's superset `persist.rs` (103 §6 rule 1); extend its v7
   legacy arm with §2's six-line rule; activate unlock tokens 4/5/6
   per §3's table (103 §2a order), restore the three Player fields in
   `app/mod.rs` beside aegis/gummy (this branch's decls
   `player.rs:2140/2146/2153` are the merge anchors).
2. Dedup §3's list (gummy/ambrosia fields + rows, the two use-cores,
   the `.wld` book-two byte, the `.plr` fruit-slot flips).
3. `.trwld`: take the 21-byte union (§4), move the six truncation
   constants, keep VERSION 18 and all three lanes' round-trip
   witnesses (re-pinned).
4. Keep this branch's consumers (§5) — they are the live ones; the
   town ladder is NOW FED by the g90 feeder (`8521139d`,
   `TownEnvFlags` + the `update_npcs` pass) — b3's phase-1 arm keeps
   the AI dispatch, its ladder call dedupes onto the feeder.
5. Update 103 §2a (tokens 4/5/6 RESERVED → LIVE g90), §4 (the six-line
   row), and 92 §1 (19 → 21, the two new offsets) at the fold.

---

## 7. Reconcile addendum (2026-11-21, second probe) — a6 + g88 latest read

Inputs re-read: `origin/task/W4-a6-crystals-powerups` tip `8ed96ee5d`
(the six use-arm commits `bb36e6ee`..`8ed96ee5` over the M2 wave-3
base), `origin/task/W4-g88-life-fruit-use` tip `6998a4ecb` (fold 5
docs, 06:16) — 23 minutes BEFORE this branch's 135 publication
(`b3b82d07`, 06:39).

### 7a. a6 — research/138 DOES NOT EXIST on the pushed tip

`git ls-tree origin/task/W4-a6-crystals-powerups docs/research/` shows
no 138 (nor any new numbered doc); the lane's only doc delta is
`docs/ANALYSIS.md` +52 ("W4-a6 — the permanent power-up consumables",
`Player.cs:45192-45423`). **No keep-one verdicts are published.** The
ANALYSIS section records the lane's own conventions (the
`powerups.rs` press-edge machine, the World-field statics standing-in,
the `notify_custom("Use")` all-rows fold, the pet-license refusal with
no resident pet, the QuickSpawnItem deposit walk) — none of it a
verdict on g90's copies.

**The overlap is real and total on the use arms.** a6's `powerups.rs`
(1387 lines, `powerup_item_check` at `:571`, dispatch added to
`world_sim/editing.rs` +30) ports: Life Crystal 29 / Mana Crystal 109
(`5eabcbe0`), Life Fruit 1291 / Demon Heart 3335 (`ea478936`), Minecart
5289 + Artisan Loaf 5326 (`bb36e6ee`), Combat Books 4382/5336
(`e15e931a`), Torch God 5043 + Satchel 5343 (`6e37aaac`), Pet Licenses
(`d46221b2`), the six shimmer permanents 5337-5342 (`f23385e5`), the
drone destroy arms (`8ed96ee5`). This branch carries the SAME use
semantics for 5326/4382/5336/5343/5337/5339/5340/5341/5342 in
`world_sim/editing.rs` (`use_artisan_loaf` `:15966`, `use_combat_book`
`:16003`, `use_peddlers_satchel` `:16041`, `use_shimmer_permanent_item`
`:16073`, dispatch `:2358-2390`).

**Cross-note verdict proposal (this lane's position, pending a6's
138):** the two machines are behaviourally equivalent at the grant
(a6: press-edge start + mid-swing `ItemTimeIsZero` grant; g90: the
press-tick fold — the same i37 convention the food/drink arms use; the
once-ever gates, the flags raised, the consumable pay and the
`ApplyItemTime` cadence all match the C# arms). At the merge ONE
machine must survive per item (the keep-one rule). This branch's
proposal: **a6's `powerups.rs` keeps the use arms it exclusively owns**
(29/109/1291/3335/5289/5043/4829/4830/4910/5451/5738 — g90 ports NONE
of those); **the overlapping nine** (5326/4382/5336/5343 + the five
shimmer) keep whichever machine the merger takes FIRST, with the other
side's dispatch deleted — g90's copies are self-contained in
`editing.rs` (four fns + one dispatch block + the witnesses), a6's are
self-contained in `powerups.rs`; neither leaks into the other's files
except the `editing.rs` dispatch (+30 a6 / the g90 block) and the
shared `World`/`Player` flag fields (identical names on both branches:
`combat_book_volume_two_was_used`, `peddlers_satchel_was_used`,
`used_aegis_crystal`, `used_arcane_crystal`, `used_galaxy_pearl`,
`used_gummy_worm`, `used_ambrosia`, `ate_artisan_bread` — the field
dedup is mechanical). **What g90 keeps regardless of the use-arm
verdict:** the `.wld`/`.trwld` book-two+satchel bytes (§4 — a6 writes
the same two `.wld` bytes at the same offsets 2036/2037 behind the
same gates 259/260, byte-identical semantics; a6 has NO `.trwld`
tail), the `.plr` halves (`player_file.rs` — a6 has no `.plr` delta),
the `.trplayer` v7 block (§1 — a6 has no sidecar delta), the net
flags3/WorldInfo bits (a6 folds all SendData out as MP-only; g90's
netglue wiring is the live one), the consumers (§5 — a6's shop +1 is
the same `shop.rs` consumer), and the town feeder (§5, `8521139d`).
**g90 does NOT port Torch God 5043** — a6 owns it (`6e37aaac`); the
`unlocked_biome_torches` consumers on this branch (craft gates
`craft.rs:658`, biome-torch placement `editing.rs:7555`, hold light
`player_lights.rs:377`) are pre-existing and stay.

### 7b. g88 — 135 NOT YET consumed (no receipt); the contract is compatible

g88's tip `6998a4ecb` (fold 5 docs) predates 135's publication; its
103/122/INDEX carry NO 135 reference and no g90 receipt beyond the
fold-4-era rows (103's carrier table lists g90 at the OLD tip
`1f1b88f0`, the bread-only v7). **The v10 status: NOT landed.** 103's
token map still marks tokens 4/5/6 (galaxy-pearl/arcane-crystal/
aegis-crystal) `RESERVED — v10 (g90b/a6 pending)`; the writer stays
**v9** (the 3-token unlock line fruit/gummy/ambrosia), the reader is
the fold-3 positional one with the v7 token-count walk intact. 122 §6's
receipt records the deliberate v9-not-v10 choice: "the writer stays 9
until the crystal tokens land then bumps v10 per 122 §3".

**Compatibility check (135 vs the v9 contract):** 135 §3's mapping
table already AGREES with 103 §2a's reserved order (pearl=4, arcane=5,
crystal=6 — the vanilla `.plr` order `:54784/:54783/:54781` after the
fruit-first deviation 122 §3.1 records); 135 §2's six-line v7
recognition rule is the piece 103's matrix still lacks (its
back-compat matrix knows only the bread-only lone-flag v7, resolved
aegis-first per 122's recommendation — 135 §2's rule subsumes that
case: a LONE 1-token line stays ambiguous-by-design, the SIX-line
block is unambiguous). No conflict: when g88 folds 135 it activates
tokens 4/5/6 from this branch's `SavedPlayer` fields (§1's table),
bumps v10, and extends the v7 legacy arm with the six-line rule. The
world-flag boundary 122 §6 honours (5336/5343 never ride line 8)
matches §4's `.trwld` tail exactly. **This branch keeps its v7/v18
shape** (the 135 header's standing decision — no rebase to v9/v10; the
merger folds per 135).

---

## 8. THE PERSIST-MERGE RECEIPT (final probe 2026-11-21 — g88 v11 canonical)

Inputs re-read at the FINAL probe: `origin/task/W4-g88-life-fruit-use`
tip **`a9f1c29d8`** (fold 7 FINAL, writer **v11**, 08:26), its
`persist.rs` + research/103 (marked FINAL) + 122; `origin/task/
W4-a6-crystals-powerups` tip **`115ce655a`** (138 FINAL, 08:59);
`origin/task/W4-g87-demon-heart-use` tip **`dfbcaf94b`** (07:19).

### 8.1 The token-order supersession — the WRITTEN order wins

135 §3 recommended 103 §2a's reserved order (pearl=4/arcane=5/
crystal=6) on the "no pushed writer ever emitted token 4+" premise.
g88 fold 6 (`c8985b534`, 07:15) then WROTE the vanilla `.plr` relative
order — **crystal=4 / arcane=5 / pearl=6** (`Player.cs:54781`/`:54783`/
`:54784`) — falsifying the premise. Both g88's 103 §8 and a6's 138 §6
adjudicate the written order binding; this lane ACCEPTS (its own
premise grants zero back-compat obligation either way, and the written
order matches f44's `used_upgrades[6]` file-order indices 0/2/3). The
§3 table's recommended column is corrected by the banner above; the
v7 six-line block's own LINE order (§1: crystal, arcane, pearl after
bread) already agrees with the written token order — only the token
NUMBERS in §2's recipe comments (`crystal -> token 6` etc.) flip to
crystal→4, arcane→5, pearl→6. g88's landed reader maps the lines
positionally to the fields, so no line-order change is needed anywhere.

### 8.2 The `.trwld` union supersession — 21 → 68 bytes, pair at 22/23

§4's three-way 21-byte union predates two pushed v18 writers:
- **f45** (`6352d3840`): a 44-byte world-identity block (138 §5).
- **g87** (`dfbcaf94b`): a 3-byte bought-pet tail (`WorldFile.cs:
  1392-1394`), same insertion point.

a6's 138 §5a publishes the exact FIVE-way byte table: g70 forest
[0..16] + g71 surface [16..19] + **g87 pets [19..22]** + **g90 books
[22..24]** + f45 identity [24..68]. This lane's pair moves 19/20 →
**22/23**; the six truncation-constant sites (§4's table) move to
**+68**; VERSION stays 18, forward-only. The g71c union owner folds
per 138 §5a; this branch keeps its own 2-byte v18 shape until the
merge (the standing decision).

### 8.3 g88 fold 7 consumption receipts — 135 §2 LANDED, §1/§3/§5 verified

Read on `a9f1c29d8`'s `persist.rs` (direct line cites):
- **§2's six-line v7 recognition rule: LANDED verbatim.** The guard
  `v7_six_line = v7_flags && nonempty.len() == 6 &&
  nonempty.iter().all(|t| t.len() == 1)` (`:463-465`) reads the block
  positionally bread/crystal/arcane/pearl/gummy/ambrosia (`:470-481`,
  `ate_artisan_bread = nonempty[0][0] == "1"` `:474`), aegis-fruit
  stays false (`:480` — g90b never wrote it). The witness
  `g90bs_six_line_v7_loads_positionally` (`:1198`) REPLACED the
  bounded-misread witness — its asserts pin all six flags + no pairs.
  **135 §2's collision is CLOSED**: a g90 lane-7 v7 file loads
  losslessly on the g88 superset reader.
- **§1's line table**: the v11 writer emits the bread flag on its own
  line after the mana pair and the six-token unlock line after dye
  (103 §2's layout); this branch's `SavedPlayer` fields + App restores
  dedup onto the superset's (103 §9 step 7).
- **§3's dedup list**: 103 §8.3 confirms "§3's dedup list matches
  §5a" — the gummy/ambrosia fields+rows, the two use-cores, the `.wld`
  book-two byte, the `.plr` fruit-slot flips all stand.
- **§4's tail**: 103 §9 step 10 carries the 21-byte union as L3 scope
  — itself one pass stale vs 138 §5a's 68-byte five-way table (§8.2
  above); the merger takes 138 §5a (later, superset).
- **§5's consumer census**: stands as this lane's recorded remainder;
  the town feeder `8521139d` is the live half.
- **v11's own addition** (not in 135): the f33 torch pair rides the
  LAST post-loadout two-token line; the f33 lone-pair v7 recognition
  (`:466-469`) coexists with §2's six-line rule — the shapes are
  disjoint (six 1-token lines vs one 2-token line), no ambiguity.

### 8.4 The stale-flag census (who read whom, when)

| Doc claim | Written | Reality at final probe | Verdict |
| --- | --- | --- | --- |
| 138 §0 "g87 = zero commits" (first pass `d7de8bd53`, 08:06) | read g87 at `a725acb04` (local main) | g87 pushed `dfbcaf94b` at **07:19** — 47 min BEFORE the 138 first pass | **138-first-pass STALE on g87**; its own §10 (FINAL, `115ce655a`) re-adjudicates — corrected in-place |
| 138 §6/§10.6 "g88 tip STILL c8985b534, six-line rule missing" (FINAL 08:59) | read g88 at fold 6 | g88 fold 7 `a9f1c29d8` landed **08:26** — 33 min BEFORE the 138 FINAL | **138-FINAL STALE on g88**: the rule IS landed (§8.3), the flag is retired by this receipt |
| 103 §9 step 10 "21-byte union" (g88 fold 7, 08:26) | predates 138 FINAL's §5a | g87 pets + f45 identity → 68 bytes | one pass stale; 138 §5a wins |
| 135 §3 "103 §2a order wins" + §4 "21-byte union" (06:39) | predates g88 fold 6 (07:15) + g87 push (07:19) | written order + five-way union | superseded by §8.1/§8.2 banners |

No `g87b-143` ref exists on origin (full `git branch -r` census at
this probe, agreeing with 138 §10's census) — the brief's
"g87b-143 re-adjudicates" resolves to **a6's own 138 §10** (the FINAL
re-adjudication of the pushed g87, same branch `115ce655a`).

---

## 9. THE 138 KEEP-ONE RECEIPTS — this lane's position after the FINAL verdicts

a6's research/138 (`d7de8bd53` first pass + `115ce655a` FINAL §10)
adjudicates every overlap. This lane's §7a proposal vs the verdicts:

| Item | 135 §7a proposal | 138 verdict | Align? |
| --- | --- | --- | --- |
| 29/109/1291 | a6 keeps (g90 ports none) | **g88 keep-own** (`crystals.rs`/`life_fruit.rs`, 103 §5a — the sidecar carrier lives there); a6's dups DROP | proposal said a6; verdict g88 — no g90 code affected, ACCEPT |
| 3335/5289/licenses | a6 keeps its exclusives | **a6 keeps** (§10.1-10.3 re-adjudicated vs the NOW-PUSHED g87: start-refusal + full spawn walk + live-scan verdicts; g87's arms DROP, its `dh` sidecar line DROP §10.4, its `.trwld` pets ride the union §10.5) | ALIGNED (g90 ports none) |
| 5326 loaf | overlap — keep whichever lands first | **g90 KEEP-OWN** (`use_artisan_loaf` + the skeleton-merchant consumer; a6's dup DROP) | **g90 WINS the loaf** — this lane's use arm + shop gate + `.plr` slot are the surviving carriers |
| 4382/5336 books | overlap — keep whichever lands first | keep ONE — a6 recommended (single cluster seam) or g89e; g90's arm dedups | ACCEPT — g90's CONSUMERS stay regardless (§5: the town feeder, fishing condition, TRAINING_DAY are g90's live halves; 138 §3.5 confirms all three lanes carry TRAINING_DAY, lands once) |
| 5343 satchel | overlap — keep whichever lands first | keep ONE — a6 or g90; the shop +1 consumer identical on both | ACCEPT either; the `.trwld` satchel byte + WorldInfo net bit stay g90's carriers regardless |
| five shimmer (5337/39/40/41/42) | overlap — keep whichever lands first | **a6's `use_shimmer_permanent` machine** owns all six arms; g90's `use_shimmer_permanent_item` dedups; **g90's Player field declarations + consumer shapes are the merge anchors** (pearl mirror, regen slots, booster order — a6 fixed to match, §3.2-3.4) | ACCEPT — the arms move, the fields/consumers stay |
| 5043 torch | g90 does not port; a6 owns | a6 use arm + c36 helper + f33 event machine (three-way split) | ALIGNED |

**THE g90 CARRIER-KEEP LIST (per 138 §1/§2/§7/§8 — what survives the
merge regardless of the use-arm dedups):**
1. `use_artisan_loaf` (`editing.rs:15966`) + dispatch `:2365` — the
   5326 keep-own (138 §1 row 5326).
2. The skeleton-merchant stock gate (`shop.rs` case 20
   `ArtisanLoafWindow` `:1455-1457` + the `npcs.rs:969-972` ShopFlags
   feeder) — the consumer that WON the keep-own.
3. The `.plr` halves (`player_file.rs` — bread gate 256 + the five
   unlock bools; dedup onto f44's walk/array per 132 §3-L1).
4. The `.trplayer` v7 six-line block as the FOLD SOURCE (§1 — already
   folded into g88 v10/v11; the branch's own writer retires at merge).
5. The `.trwld` v18 book-two/satchel pair — union offsets **22/23**
   (138 §5a; a6 has no `.trwld` tail).
6. The `.wld` book-two/satchel bytes 2036/2037 — dedup onto f45's
   superset block (138 §4; same fields/gates/offsets, byte-identical).
7. The net carriers: flags3 bits 0/2/3/4/5/6 + WorldInfo
   bitsByte9[0]/bitsByte11[6/7] (`netglue.rs` — a6 folds all SendData
   out as MP-only; g90's netglue wiring is the live one).
8. The consumers (§5): the town stat-ladder feeder (`8521139d`,
   `TownEnvFlags::from_world` + the `update_npcs` pre-pass), the
   fishing `DidNotUseCombatBook` chain, TRAINING_DAY progression-40,
   the travel-shop +1 — plus the Player field declarations
   (`used_aegis_crystal`/`used_arcane_crystal`/`used_galaxy_pearl`
   after `used_ambrosia`), the `update_luck` pearl mirror, the
   `RegenInputs.aegis_crystal`/`arcane_crystal` slots, the ambrosia
   speed triple (138 §1 shimmer row: "g90's Player field declarations
   + consumer shapes are the merge anchors").
9. The loaf consumer census (`33d4b1625`): NO build-speed leg exists
   in the C# — the flag's only consumers are AdjTiles' TB=4 reach
   (`Player.cs:35671`, live via `craft.rs::tile_reach` ← both crafting
   reads) and the merchant stock gate; two live-chain witnesses
   WRITTEN NOT RUN.

**The g87-triple flag (the brief's item):** 138's FIRST pass adjudicated
3335/5289/licenses as a6-sole-claimant on a stale zero-commit g87 read;
g87's push makes them a THREE-way overlap (a6 ⨉ g87 ⨉ f44/f45 carriers).
138 §10 (FINAL) already re-adjudicated all three arms — a6 keeps, g87's
arms + `dh` sidecar DROP, g87's `.wld` flips dedup onto f45, g87's
`.trwld` pets ride the 68-byte union. This lane ports NONE of the three
items — no g90 code is affected; the receipt is recorded here so the
merger sees the full triple resolved (no g87b-143 lane exists).