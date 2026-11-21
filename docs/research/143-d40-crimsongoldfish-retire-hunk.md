# 143 — The d40 CrimsonGoldfish-465 retire hunk (W4-c4 → merger/d40b)

Status: MERGE CONTRACT — exact drop-lines, no code changes on d40's
branch from here. Author lane: W4-c4 (`task/W4-c4-style44-flying-fish`,
the keep-c4 verdict's executor), parent `91293a3d2`.

The dup-check verdict (ANALYSIS "The style-16 dup-check: keep-one
verdicts" §CrimsonGoldfish 465, commit `318e28709`): d40's 421-490 band
ports CrimsonGoldfish at protocol **465** — the SAME id c4 uses — with
two defects vs the C#:

| Arm | d40 `8fe1d6d31` | c4 | C# truth |
|---|---|---|---|
| `knock_back_resist` | **0.4** (stock fallback) | **1.0** | `NPC.cs:14483-14495` writes NO kb row → the `:8533` head reset **1f**. d40 WRONG |
| `vanilla_type` (`npc_ai_style16.rs:101-108`) | **MISSING** (only 55/58/65) | 465 added | d40's `update_swimmer` → `ai_016_step` → `vanilla_type(CrimsonGoldfish)` hits `unreachable!` → **PANIC at runtime**. c4 works |
| FindFrame | inline case-465 (`npc.rs:9716-9731`) | `style16_find_frame` case group | behaviour-identical, two copies |
| stats/sounds/immunity/sheet | 18×20 / 31 / 7 / 110 / 500, NPCHit1/NPCDeath1, `{31}`, NPC_465 | same | equal |

**Verdict: KEEP c4's implementation at the shared id 465, RETIRE d40's
duplicate.** The id stays 465 (inside d40's frozen 430-488 vanilla-free
band — 108 §3 keeps it untouched); c4's `vanilla_type` 465 row + kb-1.0
arm + `style16_find_frame` helper survive.

## The exact drop-lines (d40 tip `8fe1d6d31`, 1-indexed, inclusive)

Every line below was read from `git show
origin/task/W4-d40-npc-kinds-421-490:<path>` at the tip hash — re-verify
with `git grep -n CrimsonGoldfish <tip> -- crates` if d40b advances the
tip before the merge (the arms are stable across its batch commits).

### `crates/tr-app/src/npc.rs`

| Lines | Action | Content / replacement |
|---|---|---|
| 1157 | EDIT | comment `// W4-d40 (NPCID 421-490 roster wave): Fritz, Nailhead, CrimsonBunny, CrimsonGoldfish, Psycho.` → drop `CrimsonGoldfish, ` (c4's variant carries the doc) |
| 1167-1169 | DROP | the `/// NPCID.CrimsonGoldfish (465)…` doc pair + the `CrimsonGoldfish,` variant (c4's variant at its own site survives; its doc carries the correct kb-1.0 note) |
| 3777 | DROP | `NpcKind::CrimsonGoldfish => 0.4,` — **the defect row**; c4's `knock_back_resist` group arm (`… | NpcKind::CrimsonGoldfish => 1.0`) survives |
| 4225 | DROP | `NpcKind::CrimsonGoldfish => 7,` (defense; c4's row equal) |
| 4665 | DROP | `NpcKind::CrimsonGoldfish => 500.0,` (value; c4's row equal) |
| 6750-6751 | DROP | the `// W4-d40: CrimsonGoldfish ride the family machine.` comment + `NpcKind::CrimsonGoldfish => self.update_swimmer(world, player, rng),` (c4's group arm routes it) |
| 9716-9731 | DROP | the whole inline `NpcKind::CrimsonGoldfish => { … }` FindFrame arm (16 lines; dedupe onto c4's `style16_find_frame` case `55\|57\|58\|102\|241\|465\|592\|607` — behaviour-identical bands) |
| 19640 | DROP | `NpcKind::CrimsonGoldfish => 465,` (the G6 render-key row; c4's row equal) |
| 24737 | EDIT | `pub(super) const ALL_KINDS: [NpcKind; 326]` → **325** (the declaration length; c4's roster appends its own kinds on top at merge) |
| 25042 | EDIT | comment → drop `CrimsonGoldfish, ` |
| 25046 | DROP | `NpcKind::CrimsonGoldfish,` (the ALL_KINDS entry; c4's entry survives) |
| 35073 | EDIT | the `w4_d40_fritz_nailhead_tests` doc line → drop `CrimsonGoldfish, ` |
| 35087 | DROP | `(NpcKind::CrimsonGoldfish, 465u16, 16u8, 18.0, 20.0, 31, 7, 110, 0.4f32, 500.0),` — **the witness pins the WRONG kb**; c4's `w4_c4_the_style16_defaults_match_the_csharp_rows` pins 1.0 |
| 35110-35111 | DROP | the hit/death-sound asserts (c4's family pin covers 1/1) |
| 35114-35115 | DROP | `// The noGravity rows.` + `assert!(NpcKind::CrimsonGoldfish.no_gravity());` (the comment's only row is this one; c4's pin asserts no_gravity for all nine) |
| 35123-35124 | DROP | the `to_u16`/`from_u16` 465 asserts (c4's pin + the round-trip test cover them) |
| 35131 | DROP | the bestiary `net_id` 465 assert (c4's bestiary entry survives) |

### `crates/tr-app/src/npc/data.rs`

| Lines | Action | Content / replacement |
|---|---|---|
| 624 | EDIT | comment → drop `CrimsonGoldfish, ` |
| 628 | DROP | `NpcKind::CrimsonGoldfish => 465,` (`to_u16`; c4's arm equal — the `deny(unreachable_patterns)` guard forces the dedupe) |
| 1026 | DROP | `465 => NpcKind::CrimsonGoldfish,` (`from_u16`; same) |
| 1969-1970 | EDIT | the HitSound group's tail: line 1969 is `\| NpcKind::CrimsonBunny` (no `=>`), line 1970 carries the group terminator `\| NpcKind::CrimsonGoldfish => "NPC_Hit_1",`. Replace the pair with `\| NpcKind::CrimsonBunny => "NPC_Hit_1",` (drop the CrimsonGoldfish alternant, move the terminator onto the last surviving alternant) |
| 2342 | EDIT | same shape: replace 2341-2342 (`\| NpcKind::CrimsonBunny` + `\| NpcKind::CrimsonGoldfish => "NPC_Killed_1",`) with `\| NpcKind::CrimsonBunny => "NPC_Killed_1",` |
| 2491 | DROP | `NpcKind::CrimsonGoldfish => true,` (no_gravity; c4's group arm carries it) |
| 2788 | DROP | `NpcKind::CrimsonGoldfish => 16,` (ai_style; c4's row equal) |
| 3385-3386 | EDIT | replace the pair `NpcKind::CrimsonBunny` + `\| NpcKind::CrimsonGoldfish => (18.0, 20.0),` with `NpcKind::CrimsonBunny => (18.0, 20.0),` (the size group keeps the bunny) |
| 4104 | DROP | `NpcKind::CrimsonGoldfish => 110,` (max_hp; c4's row equal) |
| 4536 | DROP | `NpcKind::CrimsonGoldfish => 31,` (contact_damage; c4's row equal) |

### `crates/tr-app/src/npc_buffs.rs`

| Lines | Action | Content |
|---|---|---|
| 515 | DROP | `NpcKind::CrimsonGoldfish => &[31],` (c4's `{31}` group arm survives) |

### `crates/tr-app/src/app/netglue.rs`

| Lines | Action | Content |
|---|---|---|
| 3603 | EDIT | the net-seed group `Fritz \| Nailhead \| CrimsonBunny \| CrimsonGoldfish \| Psycho => Npc::new(…)`: drop the `\| crate::npc::NpcKind::CrimsonGoldfish` alternant (line 3603); c4's own seed group carries the kind |

### `crates/tr-app/src/bestiary.rs`

| Lines | Action | Content |
|---|---|---|
| 2316-2323 | DROP | the whole `BestiaryEntryDef { credit_id: "CrimsonGoldfish", net_id: 465, … name: "Vicious Goldfish" }` block (8 lines; c4's entry — same credit/net/name — survives; the ENTRIES count pin drops by 1 on d40's side) |

### `crates/tr-app/src/ui/bestiary.rs`

| Lines | Action | Content |
|---|---|---|
| 1637-1638 | EDIT | replace `(NpcKind::CrimsonBunny, _)` + `\| (NpcKind::CrimsonGoldfish, _) => (sheet_i, 20.0, 18.0, 20.0, white),` with `(NpcKind::CrimsonBunny, _) => (sheet_i, 20.0, 18.0, 20.0, white),` (c4's portrait arm carries the goldfish) |

### `crates/tr-app/src/npc_ai_style16.rs`

| Lines | Action | Content |
|---|---|---|
| — | KEEP c4's | d40's module is the pre-c4 base (header line 10 mentions 465 in the census comment only — harmless); c4's `vanilla_type` 465 row + `style16_find_frame` are the survivors. **Do NOT take d40's module version at merge** — its `vanilla_type` lacks 465 (the panic defect) |

### NOT dropped (shared / non-kind surfaces)

- `creative_sacrifices.rs:1155` `("CrimsonGoldfishBanner", 1)` — the
  journey-mode sacrifice table (base-owned, kind-independent).
- `itemdata.rs:6987` `"CrimsonGoldfishBanner"` — the generated banner
  name list (base-owned).
- `lang_tables.rs` `(465, "CrimsonGoldfish")` — the display-name table
  (base-owned; c4's branch does not carry a duplicate row — verified
  `git grep` on both tips).

## Post-drop invariants (the merger's checklist)

1. ONE `NpcKind::CrimsonGoldfish` variant (c4's), protocol 465 both
   directions, `cs_id`/`net_id` 465, kb **1.0**, `vanilla_type` 465
   present (the swimmer arm reachable — the panic defect closed).
2. `ALL_KINDS` declaration length matches the entry count (d40 side
   326→325 before the union splice; the union pin is the 108 §6 walk).
3. FindFrame: ONE arm — c4's `style16_find_frame` case group; d40's
   inline copy gone (behaviour-identical: the wet 6/12/18/24 rows 0..3,
   dry 6/12 rows 4/5, the `sprite_dir = facing` flip).
4. The bestiary ENTRIES net-id uniqueness pin (393-row union, 108 §6)
   loses d40's duplicate 465 row, keeps c4's.
5. `w4_c4_the_style16_defaults_match_the_csharp_rows` (c4, npc.rs)
   pins the corrected row set — it is the regression witness for this
   hunk. WRITTEN, NOT RUN (code-only mode on both lanes).

## Cross-references

- The verdict + arm-by-arm table: ANALYSIS "The style-16 dup-check:
  keep-one verdicts" §CrimsonGoldfish 465 (c4 `318e28709`).
- The sibling retire contracts: BloodFeeder 241→373 + FlyingFish
  354→359 (same ANALYSIS section; the FlyingFish renumber EXECUTED on
  c4 `91293a3d2`), the d42 dup-kind findings (GoldGoldfish 366 / Pupfish
  381 / Dolphin 389 / FlyingAntlion 357 / EyeballFlyingFish 363 —
  keep-d42-ids, retire-c4-rows at merge, ANALYSIS "The proto-renumber
  EXECUTION").
- The band freeze: research/108 §3 (d40's 430-488 untouched).

Code-only: every line number read from the pushed tip via git-show;
no compiler invoked, no working-tree reads of d40's checkout.
