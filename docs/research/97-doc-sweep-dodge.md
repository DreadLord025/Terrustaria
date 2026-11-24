# 97 — DOC-SWEEP lane 19: the dodge/immune family (W4-d115, 2026-11-21)

Code-only sweep on `task/W4-d115-dodge-family` (base `fff539ad`, the DONE
dodge-family branch). Five census items; the REAL gaps ported with
witnesses WRITTEN NOT RUN (no cargo on this branch, the lane's contract).
Sibling ownership: the dust/gore bursts are **W4-g17** wave C
(`player_gore_c.rs`, `3ff14a0a`, branch task/W4-g17-gore-world-sites),
the parry machine + Cross Necklace equip arm are **W4-f48**
(`64ca5fad`…`69d499c6` + `41323c7c`, branch task/W4-f48-dodge-2) — both
censused here, neither re-ported.

Commits (one per item):

| # | Item | Commit | Verdict |
| --- | --- | --- | --- |
| 1 | dodge VISUALS (dust census + afterimage port) | `709b537b` | dust g17-owned; the renderer afterimages were the real gap — ported |
| 2 | ImmunityCooldownID full table | `e486e533` | no missing C# ids; the port's collapse documented, table landed |
| 3 | CrossNecklace parry visual DRAW census | `04da95f7` | the necklace draws no shield; the Squire/Brand layer's draw halves ported |
| 4 | BrainOfConfusion buff-321 aura census | `6c5a1c57` | the buff's stat arm was MISSING — landed live; the confuse aura is the retaliate block |
| 5 | dodge stacking order verify | `fcb4b17c` | port order matches C# verbatim; the four-source + cumulative witnesses landed |

---

## 1. Dodge visuals — the dust side is g17's, the afterimages were unported

### The dust census (C# `Player.cs:38023-38165`, the four handlers)

| Handler | Dust | Gore | Net tail |
| --- | --- | --- | --- |
| `ShadowDodge` `:38023-38039` | **none** | **none** | `SendData(62, kind 2)` `:38038` |
| `BrainOfConfusionDodge` `:38046-38055` | **none** | **none** | `SendData(62, kind 4)` `:38053` |
| `DoMysticSashDodge` `:38057-38117` | 50× type-267 `NewDustDirect` + `CloneDust` pairs (`:38060-38079`) | 5× `Next(11,14)` fan, alpha 100 (`:38083-38112`) | kind 5 `:38115` |
| `NinjaDodge` `:38119-38165` | 100× type-31 jitter dusts (`:38122-38135`) | 5× `Next(61,64)` fan, no alpha (`:38136-38160`) | kind 1 `:38163` |

The sash pair per iteration: the scale argument `Next(10,26)*0.1` rides
BEFORE the `NewDustDirect` internals (type 267 hits no tweak: 6 draws),
then 3 `NextFloat`s (fadeIn / scale / colour-lerp), then the clone's own
6 — 16 draws × 50. The ninja per dust: 6 internals + 2 jitter
`Next(-20,21)` + 1 scale `Next(40)` + the `Next(2)` gate (ALWAYS drawn)
+ on hit a second `Next(40)`. Both fans share the 5-gore velocity
recipe (centre / ±1.5 offsets, per-axis `Next(-50,51)*0.01`, kicks
overwritten then ×0.4). **All of this is ported in g17's
`player_gore_c.rs`** (`mystic_sash_dodge_gores` / `ninja_dodge_gores`,
3 witnesses) — this lane adds nothing there; the dispatch wiring into
the port's handlers stays g17's follow-up (their module is
`#![allow(dead_code)]` UNWIRED, same as this lane's draw module).

### The REAL gap: the renderer afterimages (`LegacyPlayerRenderer.cs:433-467`)

Vanilla draws two ghost families around a dodging player, and neither
had a port consumer:

* **Shadow dodge** (`:433-454`): `Player.shadowDodgeCount`
  (`Player.cs:1166`, a float with NO sim writer — the renderer is its
  only writer) steps +1/frame while `shadowDodge` (cap 30), −1/frame
  otherwise (floor 0). While > 0, TWO ghosts draw at ±count on X with
  `shadow = 0.5 + Next(-10,11)*0.005` — one `Main.rand` draw per ghost,
  the +count copy first.
* **Brain of Confusion** (`:455-467`): while the 300-tick animation
  counter > 0, `lerp = GetLerpValue(300, 270, counter)` **UNclamped**
  (`Utils.cs:315-343` default), `y = Lerp(2, 120, lerp)`; only while
  `0 ≤ lerp ≤ 1` (the first 30 ticks) SIX ghosts ring the player at
  `(0, y).RotatedBy(2π·lerp·0.5 + k·π/3)`, shadow = lerp. The C# float
  accumulator steps π/3 six times.

Ported as pure fns in **`player_dodge_draw.rs`** (UNWIRED, the g17
precedent — the `entities.rs` draw wiring is the follow-up round):
`step_shadow_dodge_count`, `shadow_dodge_ghosts`, `brain_dodge_ghosts`.
The missing `Player.shadow_dodge_count` field lands (init 0.0; the sim
tick and respawn never touch it — witnessed, matching the C#'s
renderer-only ownership). 6 witnesses.

## 2. ImmunityCooldownID — the full table, no missing ids

`ID/ImmunityCooldownID.cs:74-81` defines **seven** ids + `Count = 6`
(the `hurtCooldowns` array size, `Player.cs:3160`; General's −1 rides
`immune`/`immuneTime` instead). The four `Sets` (`:49-52`) are
BoolSets with a +1-indexed array (General maps to slot 0).

| id | value | C# consumers | Port status |
| --- | --- | --- | --- |
| General | −1 | `Hurt` default `:38208`; contact `:31332`; hostile shots `Projectile.cs:14779`; PVP sends `:10260`/`:44144`/`Projectile.cs:14687` | the single `hurt_cd` lane (M4-HAZARD-1 collapse) — every ported hurt |
| TileContactDamage | 0 | spike hurt `:28981`, cactus `:45723` | `touch_damage.rs` collapses onto `hurt_cd`, `retaliate_lane = false` |
| BossNoCheese | 1 | contact kinds 396/397/398/400/401 + 636 (`:31335-31348`); shots 452/454/455/462 + 871-874/919/923/924 (`Projectile.cs:14782-14796`) | **gap recorded**: no separate counter — the C# lets these land THROUGH General i-frames (`:38208`'s per-counter flag); the port's boss lanes collapse onto General |
| LegacyUnused2 | 2 | **NONE** (the definition is the only reference) | nothing to port |
| WrongBugNet | 3 | lava-bait bug-net burn `:44723` | `bug_net.rs` `LavaHurt` collapses onto `hurt_cd`, `retaliate_lane = false` |
| Lava | 4 | lava hurt `:27708` (gated `:27679`) | the lava lane collapses, `retaliate_lane = false` |
| PaladinsShield | 5 | team-share redirect `:38361` (`dodgeable: false`) | unported (MP team damage share) |

Sets: `Retaliate` = {General, BossNoCheese, PaladinsShield} (consumed
`:38401` — the port carries membership through `retaliate_lane`);
`Counter` = {General, BossNoCheese} (`:38305` Solar Flare consume —
unported family); `TeamDamageShare` = same (`:38338` Paladin share,
MP-only); `ImmuneTimerOnlyLimitsEffects` = {PaladinsShield} (`:38210` —
a cooldown-active redirect still runs its EFFECTS; unported with the
share).

**Verdict: no missing ids** — the C# table is complete and the port's
only gap is the deliberate per-counter collapse (already recorded in
ANALYSIS "S1-E-13" deviation 4). The table + the four membership fns
land in `id_tables.rs` (`IMMUNITY_COOLDOWN_ID`, `IMMUNITY_COOLDOWN_COUNT`,
`immunity_retaliate`/`immunity_counter`/`immunity_team_damage_share`/
`immunity_timer_only_limits_effects`) with the usage census in the doc
comment. 1 witness.

## 3. CrossNecklace parry visual — the necklace draws NO shield

Census verdict: the Cross Necklace 554's only combat write is
`longInvince` (`Player.cs:15587-15590`, landed on f48 `41323c7c` — the
120/80 dodge pair + the 80/40 hurt window). Its visible half is
`neckSlot = 2` (`Item.cs:7992`) riding the **Acc_Neck layer — the whole
visible-accessory layer family is unported** (zero `Acc_*` sheets in
`assets.rs`). There is no necklace-specific parry sprite.

The "shield-raise sprite" is the **Squire/Brand shield layer**
(`PlayerDrawLayers.DrawPlayer_25_Shield` `:2954-3024`) that the f48
parry machine raises (`hasRaisableShield`: the eleven `shieldSlot` rows
`:13123-13126` + held 4760/no-866 `:13075-13078`; `TryTogglingShield`
`:31741-31764`; the 20-tick `shieldParryTimeLeft` window). The machine
is f48-owned; its DRAW halves were unported on every branch and land
here as pure fns in `player_dodge_draw.rs`:

* the slot pick — `UpdateVisibleAccessories` `:36770-36777` (the
  `eocDash > 0` twin `:36762-36769` folds into the same predicate):
  first accessory with `shieldSlot != -1` while raised and nothing
  drawn (`raised_shield_slot`);
* the frame adjust — `:2965-2976`: the body frame's width swaps to the
  Acc_Shield texture width, the origin mirrors on a horizontal flip.
  The decompiled `bodyVect.X += bodyFrame.Width − tex.Width` (`:2971`)
  runs AFTER the width assignment so the add is always zero — a
  decompilation artifact of the original's old-width capture, recorded
  (`shield_layer_frame`);
* the anchor — `:2992`'s int-truncated position + bodyPosition + frame
  centre (`shield_layer_anchor`);
* the pulses — raise lift −4·gravDir (`:2961-2964`), the orbit
  `2.5 + 1.5·sin(2πt)` ×4 ghosts with colour `0.45 − sin·0.15` (A=0),
  the breathe `0.5 + 0.5·sin(πt)` (`:2980-3001`), the parry flash ghost
  `timeLeft/20` fade / `1.5·num4` push / `1 + num5` scale
  (`:3004-3018`) — all in item 1's commit (`shield_raise_lift`,
  `shield_raise_pulses`, `shield_parry_ghost`).

The body-frame row 10 (`shieldRaised → bodyFrame.Y = Height·10`,
`:36481-36484`) was ALREADY ported (`player.rs` `update_frames`, the
`:36489-36492` citation — witnessed at the frame test). 3 witnesses
this item (9 total across the draw module).

## 4. BrainOfConfusion — the confuse aura is the retaliate block; buff 321's stat arm was MISSING

Two separate mechanisms, censused apart:

* **The NPC-confuse aura** (`Player.cs:38403-38431`, inside `Hurt`'s
  `Sets.Retaliate` gate): on an ACCEPTED hit (not a dodge), per live
  hostile NPC `Next(500) < 300 + num2·2`, radius
  `Next(200 + num2/2, 301 + num2·2)` with the three compression steps
  (>500 ×0.75 excess, >700 ×0.5, >900 ×0.25), `AddBuff(31 Confused,
  Next(90 + num2/3, 300 + num2/2))` inside the radius, then the dart
  565 (`Centre.X + Next(-40,40)`, `Centre.Y − Next(20,60)`,
  `velocity·0.3`, damage 0). **Already ported** —
  `hurt_modifiers.rs::drain_retaliate` (S1-C-7 wave B), verbatim order
  and draws. The dart's AI (`Projectile.cs:6161-6171` SetDefaults:
  28×28, aiStyle 104, penetrate −1, alpha 255, scale 0.8; the AI
  `:33478-33511` fade-in/out + frame cycle + drift) is an unported
  projectile family — the spawn rides the port's raw pool, recorded.
* **Buff 321 itself** (BrainOfConfusionBuff, the dodge's own 240-tick
  cooldown, `:38052`): its `UpdateBuffs` arm (`:9940-9947`) grants
  **+10 melee/ranged/magic crit + 0.10 minionDamage** per slot — and
  the port had NO consumer (the buff only gated the next dodge roll).
  **The real gap.** Landed live in `player.rs`'s buff scan (the 59-arm
  neighbour) + mirrored in `buff_effects.rs`'s `DebuffEffects` table
  (the S1-E-7 convention). The buff's other table rows were already
  in: `Main.debuff[321]` (`buffs.rs` DEBUFF_IDS), NurseCannotRemove
  (`buffs.rs`), the name row (`lang_tables.rs:1098`).
  `Main.buffNoSave[321]` (`Main.cs:9049`) has no port consumer — the
  player file writes no live buffs at all (`player_file.rs` S1-J-5),
  so the row is moot until buff persistence lands.

3 witnesses (the table arm + per-slot stacking; the live dodge → scan
→ stats → expiry cycle).

## 5. Dodge stacking order — VERIFIED, port matches C# verbatim

C# `Player.cs:38214-38236` (the `whoAmI == myPlayer & dodgeable` block):

```
sash:  mysticSashDodge && Next(10) == 0            -> DoMysticSashDodge, return 0
belt:  blackBelt && Next(10) == 0                  -> NinjaDodge, return 0
brain: brainOfConfusionItem != null && !IsAir
       && Next(6) == 0 && FindBuffIndex(321) == -1 -> BrainOfConfusionDodge, return 0
shadow: shadowDodge                                -> ShadowDodge, return 0  (no roll)
```

Port `player.rs::hurt_inner` (`:4408-4455` region): identical order,
identical gates, the brain's `Next(6)` BEFORE the 321 check (the C#
`&&` left-to-right), each early return short-circuits the later draws,
shadow draws nothing. The existing `dw_1_6` witnesses covered the
three-source draw order + the first-source short-circuit; the missing
halves landed:

* **the four-source shadow fallback** — all four armed, rolls 1/1/1
  miss the three Nexts, the shadow tail dodges with zero further
  draws, queues kind 2, arms the 1800 cooldown;
* **the cumulative chance** — the roll tree walked on the port's real
  gate chain: belt 1/10; belt+brain `1 − (9/10)(5/6) = 1/4` (45/60
  hits land); sash+belt+brain `1 − (9/10)²(5/6) = 0.325` (405/600);
  the four-source stack dodges 100% while buff 59 lives. NOT additive
  (10% + 10% + 16.7% would overshoot);
* **the brain gate skips only the brain** — with 321 active the
  `Next(6)` is still consumed, the shadow tail behind it still fires
  (kind 2, not 4), and no second 321 slot is added.

3 witnesses. No code change to the roll block itself — the verify
found no defect.

---

## Remainder (recorded, not this lane's)

* The `player_dodge_draw.rs` draw wiring into `render/entities.rs`
  (the renderer runs per-frame outside the fixed step; the
  `shadow_dodge_count` step needs a draw-time call site) — the
  follow-up round, same as g17's burst dispatch.
* The Acc_Neck / Acc_Shield / visible-accessory layer family + the
  `Acc_*` atlas sheets — unported render family (the shield layer's
  maths are ready for it).
* BossNoCheese's separate counter (land-through-General semantics) +
  PaladinsShield's effects-only arm + the Solar Flare `Sets.Counter`
  consume — the per-counter array un-collapse, MP/boss lanes.
* The dart 565's aiStyle-104 machine + `Main.buffNoSave` persistence.
* g17's dust/gore dispatch wiring + f48's parry machine merge — both
  sibling branches, censused not touched.
