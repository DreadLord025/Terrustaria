# 355 — W4-d11 RETIRE FINAL: the band-101-110 closeout + the family declaration

> **Chain-banner: 245 → 355** (registry-150 §9 UNIFIED FINAL, d54 tip
> `e9ac39459`, §9.2 row 355). The original 245 self-pick lost the key:
> **g48 WINS 245** (pre-publish `2b61530b4` 14:30:16, §7.1/§8.2 — the key
> stays winner-consumed). d11 takes the Lead-assigned **355**; no self-picks.

Branch `task/W4-d11-proj-ai-101-110`, worktree TR-W4-d11. Number per the d54
collision registry (`150` §9.2 row 355): the original RETIRE-tip full scan
(362 `task/W4-*` remote tips, `git ls-tree docs/research/` per tip) picked 245
as first-free — 244 taken (f39 `8704691a3` + g48 `2b61530b4`), 246/247 claimed
mid-round by d10/g40/g43 — but the scan missed g48's PRE-PUBLISH 244/245 wins
(`2b61530b4` 14:30:16); §9 adjudicates 245 = g48 WINS and assigns d11 **355**.
The census doc renumbered
`26d` → **164** per the registry's §1 assignment (d12 wins the 26d key).

Code-only round: witnesses WRITTEN NOT RUN, no cargo/tests/rustfmt/graphify
invoked; every claim below carries a git/C# receipt re-derived this round.

## 1. Git-log census — the band 101-110 + the nebula-sweep landing state

`origin/main` at the scan: `051600da8` (the S1-L-11g recipe-freeze merge).

- `git ls-tree origin/main crates/tr-app/src/projectile_ai_101_110.rs` →
  EMPTY (the band module does not exist on main).
- `git grep -c "ai_medusa_ray|ai_tower_orb|ai_nurse_syringe" origin/main --
  projectile.rs projectile_ai_101_110.rs` → exit 1 (zero hits).
- `git log origin/main --grep=nebula -i` → the i40 wings (`c74133dd3`), the
  ManaV2 lanes, the pillar shields — NO band-101-110 or nebula-set commit.

**Verdict: the nebula-sweep has NOT landed on main.** It is complete ON THIS
BRANCH: producer `121672908` (the fields + the set arm + the `canGhostHeal`
gate + the `:13826-13830` proc + the lifecycle copies), completion
`ab42de2d7` (the `wants_nebula_pickup` drain + `spawn_nebula_pickup` + the
`aiStyle != 59` gate fix + 3 witnesses), census `2894f6f7d` (the dedupe
notes). The branch is the sole carrier of the whole band; nothing to reland.

The band's 19 commits (`git log origin/main..HEAD`): the ten style ports
(`6aca9b7f`…`28d3b2ff`), the two witness fixes (`be58df9e`, `cdd2b937`), the
census (`e0395afc`), the DOC-SWEEP wave (`eae7b5097` 619 kind, `af760b3b4`
582 latch, `c1e686f26` 584 heal-text, `121672908`+`ab42de2d7` nebula,
`2894f6f7d` census update) + this RETIRE round.

## 2. d77c cues — 539 id-4 style-7 + 576 armed on d11: compose CONFIRMED

d77 tip `5f10560b2` (the Kill-host wave `0573b70f2` + the doc `5f10560b2`):

- **539**: `kill_tail_cue` (d77 projectile.rs:20211) arm
  `539 => Some((4, 7, center.x - 40.0, center.y - 40.0, 1.0))`. C# re-read
  `Projectile.cs:72565-72571`: `position = Center; width = height = 80;
  Center = position; Damage(); PlaySound(4, (int)position.X, (int)position.Y,
  7)` — the play is AFTER the 80-box recentre, so centre−40 is the derived
  read (the 661/659 no-restore shape). The id-4 int facade gains style 7:
  d77 audio.rs:154-155 (`("NPC_Killed_7", 4)` bank row, asset verified) +
  :1443 (the `7 => "NPC_Killed_7"` arm) + the :3943-3955 witness. The
  resize/Damage host is the unported state remainder (d11's aiStyle-102 port
  is the flight machine only) — the cue composes onto d11's row regardless.
- **576**: the common-bullet list arm `14 | 20 | … | 576 | 968 => Some((2,
  10, pos.x, pos.y, 1.0))` (d77 projectile.rs:20440). C# re-read `:76038`:
  the 24-kind `else if` arm DOES name 576 (`this.type == 576`), Item10 at
  `base.position` — the pre-hook top-left read is exact.
- **d11 side**: zero `kill_tail_cue` hits, zero 539/576 cue pushes (the band
  module `projectile_ai_101_110.rs` carries no sound seam; the pool's
  `wants_sound` sites are the pre-existing lanes'). d11's rows are exactly
  what d77's comments cite: 539 `data.rs:2508` / 576 `:2517` (`e0395afc`).
- **f20 chain**: `7d6e2b72` ports the 576 AI_001 overlay (`:55361-55365` the
  ai1 Item12 latch + `:55635-55649` the fade-15 switch) into
  `projectile_ai001_e.rs`, ROW-GATED INERT until d11 merges (f20 tip
  `0883a80bc` seals the verify family; the overlay stays inert-gated).

**Compose verdict**: d11 row → f20 AI_001 overlay → d77 Kill-tail cue, three
disjoint layers, zero competing writes. Merge keeps all three; nothing drops
on d11's side. The 539's 80-box resize/Damage host + the 4+20+20+10 dust
census stay the recorded remainder (g65's dust lane / the Kill-host lane).

## 3. The 813 three-way keep-one (g64 × d11 × d13)

Tips: g64 `3cdc64236`, d13 `462c86097`, d11 HEAD `2894f6f7d` (+ this round).

**KEEP: g64's dust-wired `ai_vortex_ritual`** (g64 projectile.rs:3888, the
5-arg shape over the live `DustPool`/`SpawnCtx`/`World`). It is the strict
superset — re-read arm-by-arm this round:

| piece | g64 | d11 | d13 |
| --- | --- | --- | --- |
| wind-up dust (579 rings / 578 streak / 813 rings) | `trail_579`/`trail_578`/`trail_813_windup` WIRED | gates-only census | gates-only + contract |
| charge/full/dissolve 813 dust | `trail_813_charge/_full/_dissolve` WIRED | gates-only | gates-only |
| opacity tail (`:34018-34024`) | WIRED (`trail_813_opacity_tail`) | gate + pair census | census |
| wind-up scale | `scale = self.scale` (the d13 `b56db900` fix adopted — C# `:33714-33783` writes NO scale, re-read) | stale `scale = 0.0` stand-in | fixed 1.0 |
| dissolve Kill no-return | the d11 `be58df9e` fix absorbed + the `self.active &&` once-per-death head + `kill_time_left` stamp + `time_left = 0` (the d13 `a2588cd2` shape) | no-return fix, no Kill-head stamp | Kill-head shape |
| tick-90 Item8 cue (non-flag53, `:33792-33795`) | `wants_sound = (2, 8, pos)` (the d13 residual adopted) | absent | absent |
| 619 spawn point | `(int)Center` truncation (`:33884` C# verbatim re-read: `(int)base.Center.X/Y`) | top-left (stale) | untruncated centre |
| 619 kind | raw 619 intent ("kind unported" comment — STALE vs d11's landed row) | `NpcKind::BloodSquid.to_u16()` (`eae7b5097` row + AI_005 branches) | raw intent |

**DROP at merge**: d11's gates-only machine arm + d13's machine arm (both are
value-subsets of g64's stream: the gate rolls g64's emitters draw are the same
Next(4)×2 / Next(2)×2 / Next(2) if-else / Next(3)×2 + the opacity NextFloat +
circular pair, in the same C# order — g64's FINAL 1/2 already published the
adjudication `c82a17e19`/`625c449e0`). The 619 raw-id intent resolves through
d11's `NpcKind::BloodSquid` row at merge (g64 FINAL 1 records exactly this).

**THE ROW — d11-row-byte-equal EXECUTED this round.** The C# `:6282-6293`
block (re-read whole): 32×32, aiStyle 108, `friendly = true`, `alpha = 255`,
`tileCollide = false`, `ignoreWater = true`, `hostile = true`, `drawLayer = 1`;
it NEVER sets penetrate (stock 1, `:774` re-read) nor timeLeft (stock 3600).
d11's old row `-1`/`RESET_STATE` (alpha 0) was stale. This round:

- `data.rs` the shared `VORTEX_LIGHTNING | VORTEX_PORTAL | BLOOD_NAUTILUS_TEARS`
  tuple → `(32.0, 32.0, 108, 1, TIME_LEFT_DEFAULT, 0, false, false, false,
  (false, -2, false, -1, 255, 1.0))` — string-compare verified BYTE-EQUAL to
  g64 `3cdc64236` AND d13 `462c86097` (`True`/`True`).
- `data.rs` the `ignore_water` flag list gains the `:6290` entry (the same
  `matches!` line g64/d13 carry).
- The row witness re-pinned (pen 1 / alpha 255 / ignoreWater) + the new
  `w4_d11_retire_the_marker_row_is_byte_equal_across_the_three_way` witness
  (the three kinds value-identical + the exact 15-field tuple) — WRITTEN NOT
  RUN. The friendly/hostile/drawLayer fields stay the data-row deviation
  census (the row shape carries neither; drawLayer has no pool consumer).

The d11 machine's wind-up `scale = 0.0` stays AS-IS on this branch (the arm
drops wholesale at the keep-one; editing it would create a hybrid nobody
merges) — the comment now cites the g64 correction.

## 4. Band-101-110 all-armed/rowed census + remaining gaps

Dispatch (`projectile.rs`, one arm each, re-read this round): 101 `:18050`,
102 `:18074`, 103 `:18094`, 104 `:18103`, 106 `:18113`, 107 `:18127`,
109 `:18148`, 110 `:18176`, 105 `:18192`, 108 `:18261`. The
`s1_d_4c_every_rowed_style_has_exactly_one_live_arm` pin (103 arms, no
duplicates) is the standing coverage witness.

| kind | style | row | arm | state |
| --- | --- | --- | --- | --- |
| 536 MedusaHeadRay | 101 | ✓ | `ai_medusa_ray` | ARMED; the 535 head anchor waits on d10's merge (d10 tip `72431e61f` HAS the 535 row `data.rs:1806` + the 100 arm `:18339` — the `key_parent` probe resolves at merge; until then the ray dies like the C#'s failed `TryGetActive`) |
| 591 MinecartMechLaser | 101 | ✓ | same | ARMED |
| 539 StardustJellyfishSmall | 102 | ✓ `:2508` | `ai_tower_orb` | ARMED; the Kill cry is d77's (§2) |
| 573 NebulaBolt | 102 | ✓ | same | ARMED |
| 574 NebulaEye | 102 | ✓ (pen 1 fix) | same | ARMED |
| 576 NebulaLaser | 1 | ✓ `:2517` | generic aiStyle-1 + f20's overlay (inert until merge) | ARMED-ROWED; the overlay + Kill cue compose (§2) |
| 540 StardustTowerMark | 103 | ✓ (alpha 255 fix) | `ai_stardust_mark` | ARMED |
| 565 BrainOfConfusion | 104 | ✓ | `ai_brain_of_confusion` | ARMED |
| 567/568 SporeTrap | 105 | ✓ | `ai_spore_sac` | ARMED |
| 569/570/571 SporeGas | 106 | ✓ `:2542` | `ai_spore_gas` | ARMED |
| 575 NebulaSphere | 107 | ✓ | `ai_nebula_sphere` | ARMED |
| 596 DesertDjinnCurse | 107 | ✓ `:2554` | same | ARMED; keep-one = d11's machine + g64's `trail_596` bool seam (g64 FINAL pins `596 -> d11 ai_nebula_sphere 575 + the g64 bool seam`) |
| 578/579 ritual markers | 108 | ✓ (corrected §3) | `ai_vortex_ritual` | ARMED; the dust bodies are g64's superset at merge |
| 813 BloodNautilusTears | 108 | ✓ (byte-equal §3) | same | ARMED; g64's machine survives, the 619 spawn resolves through d11's kind row |
| 582 MechanicWrench | 109 | ✓ | `ai_mechanic_wrench` | ARMED; the latch landed `af760b3b4` |
| 584 NurseSyringeHeal | 110 | ✓ | `ai_nurse_syringe` | ARMED; the heal-text landed `c1e686f26` |

**Remaining gaps — all cited, none owed by this lane:**
- The 578/579 charge/full/dissolve dust arms + the 579 wind-up bodies: g64's
  >=401 axis (its machine wires the 813's; the 578/579 charge arms stay the
  recorded skip on EVERY tip — g64's own census says so).
- The 813 dust BODIES: g64's superset (this branch keeps the gates-only
  census until the keep-one).
- The 539's Kill-host resize/Damage + the 4+20+20+10 dust: the Kill-host /
  g65 lanes (the cue composes regardless, §2).
- The 535 head: d10's merge (row+arm verified present at its tip).
- The nebula lifecycle twins: the mana-family keep-one (§5).
- The `Lighting.AddLight` / dust-body render layer: the standing port-wide
  deviation (the gate rolls stay on the stream).

## 5. The nebula-sweep RETIRE — the producer/consumer split + the keep-one table

The 360-tip grep census (`set_nebula|nebula_cd|13826|wants_nebula_pickup|
can_ghost_heal` over projectile.rs/player.rs/npc.rs at every pushed tip)
finds exactly TWO carriers: **d11** (the whole producer chain) and **d6**
(the `can_ghost_heal` FIELD only). f49/f12/d117/d79 carry the lifecycle/
consume halves but ZERO producer code.

**D11-UNIQUE — LANDS WHOLESALE (the producer chain):**
- `Player.set_nebula` + `nebula_cd` (the `Player.cs:2820/:2822` fields) + the
  `(2760,2761,2762)` Benefits.Nebula set arm (`ArmorSetBonuses.cs:47-54/:481`
  — the cd decrement + the flag) + the `:18743` ResetEffects clear.
- The `Projectile.cs:13826-13830` on-hit proc (the Next(3) gate first, the
  SelectRandom pick, the two velocity draws, the `num23` hit-direction sign)
  + the `aiStyle != 59` exclusion (`flag10` `:13659` — the Spectre Wrath 356
  fix `ab42de2d7`) + the `wants_nebula_pickup` intent.
- `drain_nebula_pickups` + the world-sim drain + `spawn_nebula_pickup` (the
  DRAW-FREE explicit-velocity arm: `Item.cs:49738-49741` velocity.HasValue,
  prefix 0 `:511-514` early return re-read, ownership None → no grab delay).
- The 3 proc witnesses (`ab42de2d7`: the draw-order
  `[(1,101),(-15,16),(0,3),(0,3),(10,31),(-20,1)]` + payload + drain, the
  gate refusals, the spawn no-delay/draw-free).

**FOLD-SOURCE — DROPS at merge per the mana-family declaration (`fb771bfb8`,
the f49 MANA RETIRE bible; d11's copies verified line-by-line equivalent this
round):**

| d11 copy (player.rs) | canonical owner | receipt |
| --- | --- | --- |
| `nebula_levelup` `:4197` | f49 `player.rs:3492` (tip `0d1f941af`) | same family-clear `i--` rescan + the clamp-0..3 tier raise + the 480 grant; f49's consume arm (`world_sim/items.rs:296-313`) calls it |
| `update_buffs_nebula` `:4233` + `apply_nebula_buff_arm` `:4250` + the `173..=181` scan arm `:6891` | f12's g27 `buff_effects_orbs.rs::nebula_buffs` `:154` via `apply_orb_arm` `:288` + the `fold_orb` bridge (`buff_frame_wiring.rs:457`, tip `22ce2a41f`) | the ONE tier writer (the f49 R3 execution `8eb20d8b0` deleted f49's own twins for exactly this reason — d11's are the third copy) |
| `apply_nebula_buff_mana` `:4275` + the `update_survival` head call `:11319` + the `nebula_mana_counter` field | d117/f49 `regen.rs:374` (`RegenState.nebula_mana_counter`, tip `0dbcb0136`/`0d1f941af`) | byte-equivalent cadence re-read (`:19570-19583`: +level, >=6 → −6 +1 mana cap, zero-level zeroes); the regen-engine form is the declaration's ONE counter home |
| the `nebula_level_life/mana/damage` fields + the `:18755` zeros `:6297-6299` | f49's fields (`player.rs:858-871`) + its chained ResetEffects zero (`:6177`) | the f12 `fold_orb` bridge assigns the scan's tiers onto them |
| the `extra_regen` `+ 6 * nebula_level_life` feed `:11339` | f12's lifted `update_survival` reads the tier (the `buff_life_regen` feed the f49 R3 lift repaired) | d11's inline add would double the regen at merge |

**The `can_ghost_heal` keep-one — d6's FIELD wins.** d6 tip `174c0d541`
carries `Npc.can_ghost_heal: bool` (`npc.rs:1540`) ctor-derived (`:3047`, the
same `!matches!(ForceBubble | MartianTurret | MartianDrone)` gate, the 523
AncientDoom cited-unported note) + the field read at its proc site
(`projectile.rs:23788`). d11's METHOD (`npc.rs:4318`) is the same gate in a
different shape — two shapes cannot both land (the field + the method would
collide on the name). **Merge: keep d6's field, drop d11's method, re-point
d11's proc call `npcs[idx].can_ghost_heal()` → `npcs[idx].can_ghost_heal`.**
The style72/73 module-local `can_ghost_heal` fields (the outcome structs) are
a different type — no collision, they stand on every tip.

**The consumer side stays the siblings'** (d11 ports NONE of it): the
`PickupItem` nebula arm (`Player.cs:35045-35058`, f49 `world_sim/items.rs:296`
— the positioned case-7 grab cue on d79's facade + the `buffType` read +
`NebulaLevelup`), the 12/5 pull (`:34953-34956`), the `IsAPickup` gate (f49's
removed shape). Producer (d11) + consumer (f49/d79) are complementary.

## 6. The family declaration (the merger bible, one paragraph)

**The projectile-aiStyle-101-110 family is d11-owned wholesale** — the ten
AI arms, the 20 rows (the 576 laser row included), the band module, the 619
BloodSquid kind row + AI_005 branches, the 582 wrench latch, the 584 heal-text
seam, and the nebula-set PRODUCER chain (the set flag/cd + the on-hit proc +
the draw-free drop) land from this branch alone; no other tip claims any
piece (the 360-tip census). **The 813 machine is g64's** (the dust-wired
superset; d11/d13's arms drop, the shared row is byte-equal across the three
— executed this round). **The 596 machine is d11's** (`ai_nebula_sphere`, the
g64 `trail_596` bool seam feeds it). **The 576 overlay is f20's** (inert until
this branch's row merges) and **the 539/576 Kill cues are d77's**
(`kill_tail_cue`, the id-4 style-7 facade). **The nebula LIFECYCLE is the
mana-family's** (f49 levelup/fields/zeros + f12 g27 tier writer + d117/f49
regen cadence per `fb771bfb8`; d11's player.rs twins are fold-source). **The
`can_ghost_heal` gate is d6's field** (d11's method retires, the proc call
re-points). Gate command: `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` (judge by the `test result: ok` lines;
the BUILD lane owns the WRITTEN-NOT-RUN witness execution). No further d11
commits planned after this RETIRE round.

## 7. This round's edits (the receipts)

- `projectile/data.rs`: the shared marker row corrected (pen 1 / alpha 255,
  byte-equal g64/d13) + the `:6290` ignoreWater flag-list entry.
- `projectile.rs`: the row witness re-pinned + the new
  `w4_d11_retire_the_marker_row_is_byte_equal_across_the_three_way` witness
  (WRITTEN NOT RUN); the wind-up stand-in comment cites the g64 correction
  (the arm itself unchanged — it drops wholesale at the keep-one).
- `docs/research/26d-…` → `164-…` (the registry rename) + the RETIRE section.
- This doc (355; originally 245, renumbered per registry §9.2 row 355 — g48
  WINS 245) + the INDEX row + the ANALYSIS RETIRE section + the PORT_MAP
  addendum + the anchors canary.