# 64 — Buff-HUD family FINAL census + bible (W4-c26 compose, 2026-11-21)

Registry number **64** — the next free census slot (60-63 taken by the
item-use / weapon-data / ammo-potion / special-bullet censuses; f49's
doc-sweep lives at 85; the W4-f12 lane docs ride `W4-*` slugs, not numbers).
Branch `task/W4-c26-buff-hud`, code-only: witnesses WRITTEN NOT RUN, no
cargo/rustc/rustfmt/tests/graphify invoked.

This report is the FINAL record for the buff-HUD family: the landed
census, the f12/d72 compose rulings, the `buffFrame` NONEXISTENT ledger,
and the family bible (every C# fact the HUD reads, with line cites).

## 1. The landed census — what is on origin/main (2026-11-21 read)

| work | hash | in main? | scope |
| --- | --- | --- | --- |
| M2-BUFF-1 slot engine | 9ed5feece | yes | AddBuff/DelBuff/ClearBuff/FindBuffIndex, UpdateBuffs timer head, immune purge |
| M2-BUFF-3 DoT/cures | b97e53dbf | yes | player DoT effects, cures, demo inflictions |
| S1-E-6 buff HUD | ab702b877 (merge 792caf625) | yes | `draw_buffs` grid/alpha/duration/cancel (`Main.cs:43203-43402`) |
| S1-E-6 hover follow-up | 4604557c3 | yes | hover block + the 28-handler projectile-counter icon text |
| S1-E-8 nurse rules | 5a889e32a | yes | NurseCannotRemoveDebuff + the boss cost ladder |
| S1-P-i53 cancel census | c3492e9db (merge c405707f3) | yes | five WRONGs: dead-branch drain, gate+latch, mount refusal, hide-misc, clear_buff |
| i19/i52 mana fields | 13a6392f1 / a54d32c7b | yes | `mana_sick_reduction` / `mana_heat` (the 94/396 tooltip fields) |
| W4-c16/c17/c18/d71/d72/d73/d74 waves B-H | ffffa3b49 / 357dabedd / cef7e5aa6 / 315dec645 / cf96c8c1c / 33c38000a / 3a80d60f2 | NO (branches) | the UpdateBuffs effect blocks, gathered by f12 |
| W4-f12 (both refs, same tip) | 22ce2a41f | NO (branch) | the 270-id wiring + the HUD final census + closeout + retire receipts |
| W4-g26 buff-wiring-verify | ac5d1874e | NO (branch) | verify lane |
| W4-c26 buff HUD verbatim round | 43245a5f3 (+4c3ddded0) | NO (this branch) | icon-text split, percent arms, placeholder, strict seam |

`git merge-base --is-ancestor` exit codes: every W4 buff branch above = 1
(not in main); the S1 rows are ancestors (0). f12's two refs
(`W4-f12-buff-effects`, `W4-f12-buff-frame-wiring`) point at the SAME tip
22ce2a41f — one lane, two names.

## 2. The f12 compose (270 ids / 400 icons) — rulings applied

`origin/main...origin/task/W4-f12-buff-effects` = 32 files, +9196/−78:
the eight `buff_effects_*` wave modules + `buff_frame_wiring.rs` (935
lines, the ONE slot-ordered UpdateBuffs dispatch) + `buff_inferno_aura.rs`
+ the player.rs/world_sim seams + the HUD final-census edits to
`ui/resources.rs` + the closeout/sweep/g27 docs + the sweep script.

The c26 and f12 lanes fixed the SAME three HUD drifts independently
(the icon-text font/scale/tint, the 94/396 percent arms). The compose
(this branch, merge commit d5e822240) ruled:

1. **ONE `buff_tooltip`** — the c26 text-then-percent shape kept (the
   WellFed expert swap mutates `text`, the percent arms append to it —
   the C# switch's fall-through, `Main.cs:43257-43288`). The 396 ceil is
   `(p.mana_heat * 100.0f32) as f64).ceil()` — `Math.Ceiling` takes a
   double, so the f32 product is promoted BEFORE the ceiling (the f12
   shape ceiled in f32; identical for every value the field takes, but
   the f64 form is the literal signature). The auto-merge had spliced
   BOTH matches (a double "26%26%" append) — resolved to one, guarded by
   `w4_c26_compose_the_percent_suffix_is_appended_exactly_once`.
2. **`buff_icon_text` split kept** — the f12 inline selection in
   `draw_buffs` is byte-for-byte the same body (counter first, then the
   >2-tick duration at `t / 60`); the named fn stays so the gates
   witness without a renderer.
3. **The c26 grey 32×32 placeholder kept** — f12 still `continue`d on a
   missing sheet (the slot vanishes AND skips hover/cancel); the c26
   stand-in keeps the slot alive (documented deviation, ANALYSIS "The
   buff HUD verbatim round").
4. **The `ui::in_rect_strict` seam kept** — f12 carried the private
   four-comparison copy; identical semantics (`:43329` open on every
   edge), one copy per the house rule.
5. **ONE composed percent witness** —
   `w4_c26_the_mana_percent_arms_append_the_player_computed_text`
   carries both lanes' edges (26/26/13/13/1% truncation floors;
   13/11/50/0% ceiling bumps). The f12 witness
   `w4_f12_final_tooltip_…` retired into it. **Fixture correction**:
   both lanes registered `BuffDescription.ManaSickness/ManaHeat` under
   `Items.json` — wrong file. The retail rows live in
   `Terraria.Localization.Content.en-US.Game.json` (`BuffDescription`
   category opens :996; `ManaSickness` :1083 "Magic damage reduced by ",
   `ManaHeat` :1342 "Magic damage increased by "); `Items.json`'s
   `BuffDescription` (:141-144) carries ONLY the three
   `WellFed{,2,3}_Expert` rows. The composed witness registers Game.json
   with the retail strings. (The manager keeps per-source rows and
   reloads in sorted file order — Game before Items — so the WellFed
   expert fixture in Items.json still wins for its three keys.)
6. **f12's other HUD witnesses kept verbatim** —
   `w4_f12_final_buff_time_the_unit_boundaries_and_the_two_tick_gate`
   and `w4_f12_final_icon_census_the_gates_are_the_csharp_sets` (one
   comment corrected: 72 is **Midas** (`BuffID.cs:315`), not Tipsy
   (25, `:221`) — the assertion itself was right).
7. **Docs keep-both composed** — anchors (the c26 row + all 14 f12/wave
   rows), PORT_MAP (one S1-E-6 row, both evidence tails), ROADMAP (one
   S1-E-6 row: c26 tail + f12 final/closeout/retire tails + the compose
   note), ANALYSIS (both sections + the compose note in "The buff
   hover's two restored arms").

## 3. The buffFrame NONEXISTENT census — the C#-facts ledger

**Verdict: `Main.buffFrame` / `buffFrameCounter` DO NOT EXIST.**
Case-insensitive grep for `buffFrame` over the whole `decompiled/` tree
(*.cs): **zero hits** (re-run on this tree 2026-11-21, independently of
f12's closeout item 2 — same verdict). The Rust port has no `buff_frame`
symbol either; the only `BuffFrame*` identifiers are the f12 wiring's
`BuffFrameInputs`/`update_buff_frame` (the UpdateBuffs TICK pass, not an
icon animation). The lane-name premise "buff-frame wiring" refers to
the UpdateBuffs frame (the per-tick pass), never to icon frames.

The ledger (every fact re-read in ≤100-line windows on this tree):

| fact | C# cite | value |
| --- | --- | --- |
| icon draw is the FULL sheet, static | `Main.cs:43315` | `Rectangle(0, 0, Width(), Height())`, rotation 0, scale 1 — no frame selection of any kind |
| icon count loaded | `AssetInitializer.cs:418-420` | `for num53 = 1; num53 < BuffID.Count` → `Images\Buff_<id>` — ids 1..400, **400 static sheets** |
| id-space size | `BuffID.cs:997` | `BuffID.Count = 401` (id 0 = none) |
| alpha array | `Main.cs:925` decl, `:66775` init | `float[BuffID.Count]` = 401 entries |
| slot count | `Player.cs:57106` | `maxBuffs = 44` (port `buffs.rs:21 MAX_BUFFS = 44`) |
| grid | `Main.cs:43206-43221` | 11 per row, x = 32 + col·38, y = 76 + row·50, packed occupied slots |
| empty-slot alpha reset | `Main.cs:43227` | `buffAlpha[i] = 0.4f` when `buffType[i] <= 0` |
| icon tint | `Main.cs:43314` | `Color(a,a,a,a)` — the SAME quadruple reused for the text at `:43327` |
| icon text | `Main.cs:43316-43327` | handler counter first (`BuffTextHandlers`), else `TryGetBuffTime && > 2` → `LocalizedDuration(buffTimeValue / 60)`; `FontAssets.ItemStack`, scale **0.8f**, at `(x, y + Height())` |
| hover hit-test | `Main.cs:43329` | strict `<`/`>` on ALL four edges + `!PlayerInput.IgnoreMouseInterface` |
| hover alpha | `Main.cs:43332` / `:43353` | +0.1 hovered / −0.05 rest |
| alpha clamp | `Main.cs:43355-43361` | >1 → 1; `(double)a < 0.4` → 0.4 (the lower compare promotes to double) |
| cancel gate | `Main.cs:43333` | `mouseRight && mouseRightRelease` (gamepad arm :43334-43341: `mouseLeft && mouseLeftRelease && playerInventory`) |
| cancel refusal | `Main.cs:43373` | `!Main.debuff[b] && b != 60 && b != 151` |
| mount arm | `Main.cs:43375-43379` | `mount.Active && mount.CheckBuff(b)` → `TryDismount`, `flag = true` (no DelBuff) |
| hide-misc | `Main.cs:43380-43381`, `:43390-43404` | slots 0/1, `miscEquips[slot].buffType == b`, the 101/102↔27 alias |
| cancel cue | `Main.cs:43382` | `PlaySound(12)` MenuTick, BEFORE the DelBuff |
| tooltip percent 94 | `Main.cs:43282` | `text + ((int)(manaSickReduction * 100f) + 1) + "%"` — +1 AFTER the truncating cast |
| tooltip percent 396 | `Main.cs:43285` | `text + (int)(Math.Ceiling(manaHeat * 100f) - 100.0) + "%"` — f32 product, double ceiling |
| debug toggle | `Main.cs:43278-43281` | `DebugOptions.ManaPotionDelay` early-return — constants-off false |
| duration gates | `Main.cs:43295-43300` | `!vanityPet && !lightPet && !buffNoTimeDisplay && !(honeyWet && 48) && !(wet && expert && 46)`; vampire seed hides 23/24/32 under 10 ticks |
| hover name/desc | `Main.cs:43237-43238` | `Lang.GetBuffName` / `GetBuffTooltip` direct; banner 147 sets `bannerMouseOver` (:43239-43242); melee buffs pass rarity −10 (:43243-43246) |
| fullscreen-map second site | `Main.cs:41330-41344` | the same `DrawBuffIcon`, 46-px pitch, alpha floor **0.65** (:41338-41340) — rides the map tasks |
| the ONE real buff frame anim | `Player.cs:10462-10485` | Ice Barrier 62: below-half-life gate, `iceBarrierFrameCounter++` then `> 2` → reset + `iceBarrierFrame++` wrap 12; above half → `DelBuff(j); j--`. Consumer `PlayerDrawLayers` :3918-3923 (draw lane). SIM wired via the d73 boosters arm in the composed tree |

The animated-buff remainder (beetle 3-frame/orbits, solar orbits, nebula
orbs, titanium shader) is player-draw-layer work — the sim counters are
live; see `W4-f12-closeout.md` item 2 for the full table.

## 4. The d72 compose (wave F)

`origin/task/W4-d72-buff-effects-wave-f` tip **cf96c8c1c** is an
ANCESTOR of f12 (`git merge-base --is-ancestor` exit 0;
`f12..d72` = empty). Its two code blobs are byte-identical in both
trees: `buff_effects_frame.rs` = **e7fe827bc**, `potions.rs` =
**1dcc89c83**. Composing f12 therefore composes d72 whole — no separate
merge, no dup. Wave F is the UpdateBuffs FRAME (soulDrain 151 head,
DontStarve 332 re-grant, the per-tick hunger ladder call site, the
luckPotion sync diff, the lavaOpacity 0.04 fade; `Player.cs:9857-9864`
+ `:12046-12067` + `:12152-12182`), split around the wiring chain by
`buff_frame_wiring.rs`. It touches NO HUD file (`ui/resources.rs` on
d72 = main's S1-E-6 state).

## 5. The family bible — C# → Rust map (composed tree)

| C# | Rust | notes |
| --- | --- | --- |
| `DrawInterface_Resources_Buffs` grid (`Main.cs:43203-43252`) | `ui/resources.rs::draw_buffs` + `buff_slot_pos` | 11/38/50, packed, empty-reset 0.4 |
| `DrawBuffIcon` (`:43307-43368`) | `draw_buffs` body + `buff_icon_text` | static full sheet; grey 32×32 placeholder on a missing sheet (deviation); text ItemStack 0.8 slot-alpha |
| `TryGetBuffTime` (`:43291-43305`) | `try_get_buff_time` | the five gates + the vampire sub-10 |
| `GetBuffTooltip` (`:43254-43289`) | `buff_tooltip` | WellFed expert swap + the 94/396 percent arms (ONE match) |
| `Lang.LocalizedDuration` (`Lang.cs:1158-1207`) | `localized_duration` | abbreviated ceilings, strict >1-unit boundaries |
| `BuffID.Sets.BuffTextHandlers` (`BuffID.cs:54-168`) | `BUFF_TEXT_HANDLERS` (28 rows) + `buff_text` | xN = ownedProjectileCounts sum, never buff stacks |
| `TryRemovingBuff` (`:43370-43388`) | `player.rs::try_removing_buff` (tick drain) | i53: alive-player drain, gate+latch, mount refusal, PlaySound(12) |
| `CheckBuffHideMisc` (`:43390-43404`) | `player.rs::check_buff_hide_misc` | the 27 alias; write-only until UpdatePet/TogglePet |
| `MouseText_DrawBuffTooltip` (`:21345-21489`) + `MouseTextInner` plain arm (`:20218-20319`) | `draw_buff_hover` + `mouse_text_color` + `tooltip::draw_mouse_text_block` | −10 green (65,255,110); banner list (80,255,120); box/column geometry = documented shared-printer deviation; −13/−12 unreachable drops |
| `Main.buffAlpha` (`:925`/`:66775`) | `App::buff_alpha` | hover +0.1 / rest −0.05, clamp [0.4,1] |
| `TextureAssets.Buff` (`AssetInitializer.cs:418-420`) | `assets.rs::enumerate_buff_icon_ids` → `buff_icons` | enumeration-only atlas (the placeholder covers the gap) |
| `UpdateBuffs` (`Player.cs:9855-12069`, 270 ids) | `buff_frame_wiring.rs::update_buff_frame` + the eight wave modules | f12 canonical; g27 orbs blob c6bcb017b; d73 boosters blob 9d737e57e owns {62,192,321,396} |
| `AddBuff` re-grant semantics (`Player.cs:5263-5449`) | `buffs.rs::add_buff` | refresh-to-max default; ONLY 94 (cap 600) / 383 (cap 43200) additive; fed/pet/flask wipes; warmth cuts; orb/nebula ladders; mounts pin 10; 8 no-tick ids |

**Cross-tree merge obligations still open** (f12 RETIRE receipts,
`W4-f12-closeout.md` "RETIRE phase" — they ride the BUFF merge, not this
compose): d115's inline 321 drop (:6632-6648), f50's Panic-63 drop
(:7010-7013) + the `rederive_beetle_solar_buffs` fold (:4606-4708 +
:4582 + the :7451 lift + the :6203-6205 flag-lifecycle fix + the
OrbContext live-flag replacement), f49's nebula both-halves bridge
(delete :6849-6876 + :3519-3537, lift :7766, apply the published
`fold_orb` ASSIGN), then `python tools/w4_f12_closeout_sweep.py` must
print set-diff EMPTY.

## 6. Witnesses (WRITTEN NOT RUN)

Composed `ui/resources.rs::s1_e_6_tests`: the S1-E-6 base set +
`w4_c26_the_mana_percent_arms_append_the_player_computed_text` (composed,
retail Game.json fixture) + `w4_c26_compose_the_percent_suffix_is_appended_exactly_once`
(new, the double-append guard) + `w4_c26_the_icon_text_prefers_the_counter_then_the_duration`
+ `w4_c26_the_hover_name_reads_the_buff_name_row_directly` +
`w4_c26_the_cancel_hit_test_is_strict_on_every_edge` +
`w4_f12_final_buff_time_the_unit_boundaries_and_the_two_tick_gate` +
`w4_f12_final_icon_census_the_gates_are_the_csharp_sets`.
`buffs.rs`: `w4_f12_closeout_regrant_is_refresh_except_the_two_additive_ids`.
The gate conjunction stays
`app/frame/draw.rs::s1_p_i53_the_cancel_gate_fires_on_press_not_on_release`.
All f12 wave/wiring/sweep/closeout witnesses ride their modules unchanged.

RNG: untouched — the HUD draw path draws no gameplay RNG (the cancel cue
is the fixed MenuTick sound); the compose adds no sim code beyond f12's
own (whose RNG census is zero-draw per wave).
