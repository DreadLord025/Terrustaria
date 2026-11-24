# 28g — the BUFF-FAMILY SEAL (registry number 28g, 2026-11-21)

> The buff family's FINAL record: registry home **28**
> (`28-buffs-debuffs.md`), wave claims 28c/28d/28e/28f taken — this seal
> is **28g**. It supersedes the W4-f12 sweep/final/closeout/retire
> phases' MONITOR notes (their censuses + drop-hunk rulings stand, the
> stale tip-reads are corrected here) and closes the lane with the
> buff-family bible below.

## W4-f12 BUFF SEAL — the five seal items

Branch `task/W4-f12-buff-effects` (from `task/W4-f12-buff-frame-wiring`
tip 22ce2a41f). Code-only: witnesses WRITTEN, NOT RUN (no
cargo/tests/rustfmt/graphify). This seal closes the BUFF lane against the
NEW tips the RETIRE phase had not seen: f49 SEALED **0d1f941af** (R3
EXECUTED at 8eb20d8b — the RETIRE receipt-2 monitor note is superseded,
mutually acknowledged by their research/85 RT1), f50 SEALED **51eefdf84**
(the player.rs blob 156fc3fec UNCHANGED since 18eeb4f44 — the RETIRE
drop-hunks were re-pinned at that tip by their own closeout Receipt 1),
f48 **e5b66bb06** (the THIRD panic copy their census found at 69d499c6b
still stands, blob 2120682ca).

## Seal item 1 — the buffs lane is LANDED: 270 ids zero-dup, 400 icons

The lane's code half is final on this tree (inherited from
task/W4-f12-buff-frame-wiring, unchanged by the seal except witnesses):

* **270-id UpdateBuffs census, zero dup-grants.**
  `tools/w4_f12_closeout_sweep.py` RE-RUN at seal time (this tree):

  ```
  C# heads: 270 ids, 6 range heads [(95, 97), (170, 172), (98, 100), (176, 178), (173, 175), (179, 181)]
  INLINE_OWNED: 62  ACCESSORY_OWNED: 78  SUMMON_OWNED: 32
  DEBUFF_DISPATCH: 24  MISC_DISPATCH: 11  wave-E status: 37
  d73 non-orb: 4  g27 orbs: 18  mount head: 66  other lanes: 4
  missing owners: []   extra owned ids (non-mount): []
  VERDICT: set-diff EMPTY, zero dup-grants — keep-one HOLDS
  ```

  Every one of the 270 C# case ids (`Player.cs:9875-12044`) has exactly
  ONE live grant path on this tree. The cross-tree dups are merge-time
  folds (seal items 2-4), not lane gaps.
* **400 icons landed.** `assets/Content/Images` carries exactly 400
  `Buff_<id>.xnb` files (counted at seal time: **400**; `BuffID.cs` ids
  1..=400, `Count` :997 = 401). `enumerate_buff_icon_ids`
  (`assets.rs:57-73`) → `Lookups.buff_icons` packs every one; the HUD
  reader (`ui/resources.rs:586`) resolves per-slot. All 400 sheets are
  STATIC (`DrawBuffIcon` :43315 draws the full sheet — the closeout
  animation census, no icon-cycle exists). The final-phase icon census
  (ANALYSIS "The buff-HUD final census") pinned the grid/alpha/text 1:1.
* Seal witnesses (assets.rs, written not run):
  `w4_f12_seal_buff_icon_enumeration_sorted_deduped` (the Buff_-prefix
  enumeration contract) + `w4_f12_seal_buff_icons_enumerate_the_full_400`
  (the ids are exactly 1..=400 and every one packs).

## Seal item 2 — the f12 bridge: 3 ASSIGNs + items.rs (the f49-mana-seal)

**The RETIRE receipt-2 monitor is SUPERSEDED — mutually.** f49's SEALED
tip **0d1f941af** (research/85 RT1-RT4; the code half FINAL at cded6714,
R3 EXECUTED at **8eb20d8b**) carries:

* the three scan arms GONE — their player.rs:6874-6888 is the
  R3-EXECUTED tombstone inside the buff loop's `_ => {}` tail;
  `update_buffs_nebula_buffs` deleted (doc-tombstone :3523-3531 cites
  the g27 home);
* the bridge stub LANDED — `Player::fold_orb_nebula_bridge(mana, life,
  damage)` at their player.rs:3547-3551, three sequential ASSIGNs;
* the lift LANDED — `update_survival` at the seam
  (world_sim/player.rs:1204, after `update_summon_buff_arms` :1185),
  NOT inside `update_with_grapple` (their :7778-7788 comment records
  the lift + the f12 `buff_life_regen` repair); the standalone
  `Player::update` wrapper re-adds it (:5763) for no-wiring callers;
* kept per the contract: `nebula_levelup` (:3492), both ResetEffects
  zeros (:6182-6184), `world_sim/items.rs` wholesale.

Their RT1 explicitly acknowledges f12-writer-canonical (blob c6bcb017b
re-verified IDENTICAL at our tip and g27 tip 1cd7ac367 — re-verified
AGAIN at seal time: `git rev-parse` both = `c6bcb017b3473be3…`).

**The SEAL bridge ruling — the stub form, all three tiers.** Their RT1
permits two forms (the mana-only published hunk, or the stub). The seal
takes the STUB form — f49's own field docs (:863-871) keep the Life and
Damage Player twins for their consumers, and the stub bridges all three
in one call, ASSIGN-shaped:

```rust
fn fold_orb(p: &mut Player, e: &OrbEffects) {
    p.beetle_buff |= e.beetle_buff;
    // R3 bridge (f49 research/85 RT1): the ONE tier writer is g27
    // `nebula_buffs` inside `apply_orb_arm` (OrbScanState, zero-seeded
    // every tick); the stub's three ASSIGNs are the `:12072` shape.
    p.fold_orb_nebula_bridge(
        p.orb_scan.nebula_level_mana,
        p.orb_scan.nebula_level_life,
        p.orb_scan.nebula_level_damage,
    );
    p.melee_damage += e.melee_damage;
    // … the four-class adds + buff_life_regen stay as published.
}
```

(The three field reads evaluate before the `&mut p` call — borrow-clean,
their RT1 note. The hunk lands with the merge: the stub + the Player
fields exist only on f49's tree. `fold_orb` on THIS tree is unchanged —
`buff_frame_wiring.rs:457-465`, the call site :299.)

**The extra_regen SUM is the merge-keep (:11492).** This tree's read is
`extra_regen: self.accessory_life_regen + self.buff_life_regen`
(player.rs:11492); f49's SEALED tree reads the accessory half only
(:11345). Their RT1 remainder item 4 rules the merge keeps the SUM —
the lift makes the fold visible to the read, the sum carries the Nebula
Life +6 (`:10451`) and every wave-fold regen add into the engine
(`regen.rs:176` `state.life_regen += in_.extra_regen`).

**items.rs wholesale → f49 (seal item 4's merger action).** This tree's
`world_sim/items.rs` is UNTOUCHED vs origin/main (`git diff
origin/main...HEAD --stat -- crates/tr-app/src/world_sim/items.rs`
empty); f49's R3 step 5 keeps THEIR items.rs wholesale (the IsAPickup
gate removal + the nebula/4143 consume arms + the 12/5 pull; their RT2
adds the d79 pool-facade rewire). The 94-drink grant resolves to ONE
line: this tree's `potions.rs:216` (`add_buff(MANA_SICKNESS,
MANA_SICK_TIME)` = `AddBuff(94, 300)`, `:51938`) is the same grant as
their `apply_life_and_or_mana` — the merge keeps one.

**The merged-tick single chain (their RT3 bible, f12-side confirmed):**
timer head → both resets (Player ResetEffects + `orb_scan.reset_effects`
:6199) → UpdateEquips → `update_buff_frame` scan + fold_orb bridge →
the LIFTED `update_survival` (+ the d79 refill capture) → the refill
drain → projectiles prism pass-tail → items consume arms →
`nebula_levelup` (the same-tick pickup hint — the C# runs GrabItems
AFTER the scan).

Seal witnesses (written not run):
`w4_f12_seal_bridge_scan_tiers_are_assign_shaped_for_the_f49_stub`
(buff_frame_wiring.rs — all three tiers step ONCE at the time-2 edge,
re-arm 480, the +6 Life fold lands, the head reset + empty bar leaves
all three 0) +
`w4_f12_seal_extra_regen_sums_the_accessory_and_buff_fold` (player.rs —
the :11492 sum feeds 2 + 6 = 8 into `regen.life_regen` on a fresh ramp).

## Seal item 3 — Panic-63 canonical: the f50/f48 THIRD copies drop

**The dispatch-keeper ruling stands and now covers THREE trees.** The
wiring's DEBUFF_DISPATCH 63 arm (`buff_effects.rs:149`, the
`Player.cs:10130-10133` `moveSpeed++` — re-read at seal time) is the ONE
reader. Both other copies are inline twins in the player.rs effect
match; merged as-is each would add +1 moveSpeed/tick (+3 total).

**f50 drop-hunks, RE-PINNED at their SEALED tip 51eefdf84** (the
player.rs blob `156fc3fec` is byte-identical to 18eeb4f44 — verified
`git rev-parse` both; their own closeout Receipt 1 already re-pinned
these, the seal confirms them):

* DELETE **:7165-7168** — the 3-line W4-f50 comment + the
  `63 => self.move_speed += 1.0,` row (between the Swiftness-3 row
  :7164 and the Mining-104 comment :7169).
* DELETE **:4726-4828** — the 7-line doc comment + the whole
  `pub fn rederive_beetle_solar_buffs` body (fn at :4733).
* DELETE **:4642** — the `self.rederive_beetle_solar_buffs();` call;
  `update_beetle_solar_sets` (:4641) keeps ONLY the machines:
  `apply_set_bonus_solar` (:4482-4515),
  `apply_set_bonus_beetle_defense` (:4520-4540),
  `apply_set_bonus_beetle_damage` (:4548-4589),
  `update_armor_sets_always_beetle` (:4666+, the dev-3 orbit draws
  included — their bible rules keep whole),
  `update_armor_sets_always_solar` (:4625-4629) + the dev-4
  Necro/Frost dust tail (:4654+).
* REWORD **:4631-4640** (wrapper doc — the "idempotent per tick" claim
  is FALSE vs g27: the Might stats + the delete/rescan walk
  double-apply) + **:7599-7606** (call-site comment).
* LIFT **:7607** — `self.update_beetle_solar_sets(rng);` moves out of
  `update_with_grapple` to the wiring seam AFTER `update_buff_frame`
  (this tree world_sim/player.rs:1230; C# order UpdateBuffs :25566 →
  UpdateArmorSets :25685). Same seam as f49's lifted `update_survival`
  — machines BEFORE the regen read (C# :25685 < :25977).
* FLAG-LIFECYCLE FIX **:6352-6354** — their ResetEffects clears
  `beetle_defense/beetle_offense/set_solar` PRE-scan; the C# clears at
  `Player.Update` :25618-25620 AFTER UpdateBuffs. Move the three clears
  post-scan AND replace this wiring's `OrbContext::default()` stub
  (buff_frame_wiring.rs:153-155) with the live
  `set_solar/beetle_defense/beetle_offense` flags — without both, the
  scan reads all-false and deletes every orb buff.
* RETARGET the two witnesses: `w4_f50_the_panic_buff_raises_the_move_speed`
  (:16199-16215, drives `p.update()` — needs the wiring seam rig or
  retire in favour of our closeout witness) +
  `w4_f50_the_beetle_solar_buff_ladder_rederives_the_counts`
  (:16342-16379, calls the deleted fn 4× — re-derive against g27
  `apply_orb_arm` or retire; the g27 module's 22 witnesses pin the arms).
* KEEP: the hurt-head grants 63/205 (:4901-4906), the `panic` field
  (:1486-1493) + equip rows (:6106-6123) + clear (:6735-6737), the
  machines, the 3-lane drains, the solar/beetle hurt cuts, the dev-3
  beetle orb-orbit draw layer (`render/player_layers.rs::beetle_orb_layers`
  — their 18eeb4f44 landed it; our closeout item-2 "UNPORTED" note is
  superseded), the dust request flags. The `beetle_orbs/solar_shields/
  beetle_buff` Player fields (:1504/:1508/:1511) fold onto this tree's
  `orb_scan` counters + `beetle_buff` at the merge (the g27 scan is the
  writer; their machines read the same values through the seam).

**f48 THIRD-copy drop-hunks, pinned at their tip e5b66bb06** (blob
`2120682ca`; f50's sub-receipt read 69d499c6b blob f74877839 — the
ranges shifted +14/+20 by their three later commits, re-pinned here):

* DELETE **:4528-4545** — the panic-grant comment (:4528-4533) +
  `if self.panic` block (:4534-4536) + the squire-T2 comment
  (:4537-4542) + `if self.set_squire_t2` block (:4543-4545).
  f50-canonical hurt head (their :4901-4906).
* DELETE **:1463-1471** — the `panic` field doc + field (f50's
  :1486-1493 stays). DELETE **:2818** — the spawn `panic: false`.
* EQUIP ROWS **:5663-5678** — drop the `panic` halves ONLY: the 1578
  arm keeps `honey_comb = true` (drop :5668 + the comment's panic
  half), DELETE the bare 1290 row :5670-5671, the 6187 arm keeps
  `long_invince = true` (drop :5676). f50's rows (:6106-6123) own all
  three grants.
* DELETE **:6411-6412** — the ResetEffects `panic = false` comment +
  clear (f50's :6735-6737 stays).
* DELETE **:6829-6834** — the inline 63 comment + row (f12 canonical).
* RETARGET the panic witnesses: `w4_f48_the_panic_necklaces_arm_the_flag_and_clear`
  (:15812-15831), `w4_f48_the_accepted_hit_grants_panic_63_for_480_ticks`
  (:15833-15849), `w4_f48_parry_the_squire_set_grants_ballista_panic_205_on_the_accepted_hit`
  (:15850-15876), `w4_f48_a_dodged_hit_grants_no_panic_and_draws_no_extra_roll`
  (:15877-15897) — all compose 1:1 with f50's head (same post-dodge
  site); `w4_f48_the_panic_buff_doubles_the_move_speed_until_expiry`
  (:15898-15918) drives `p.update()` — retarget onto the wiring seam
  or retire. KEEP: the dodge block itself (f48's lane, on main), the
  `long_invince` field/consumers (main's), the DD2 `add_buff(205, 3600)`
  crystal-lane grant (a DIFFERENT site, lands clean), the
  `ballista_panic` 205 READER row (:6944 — also on this tree :6973 and
  main :6674; NOT a dup, the reader is inline-owned on every tree).

Seal witness (written not run):
`w4_f12_seal_panic_63_and_the_ladder_keep_one_dispatch_owner`
(buff_frame_wiring.rs — 63 is DEBUFF_DISPATCH-only, the 9 ladder ids
are g27-only, the single +1 through the wiring).

## Seal item 4 — the owed merger-actions (the BUFF merge checklist)

Everything the BUFF merge still owes, in order, with the tip each
action reads (the RETIRE receipt-4 paragraph stands; this is its
seal-time expansion — f49's half B is ALREADY EXECUTED on their tree,
so the merger applies only the f12-side halves + the f50/f48/d115
drops):

1. **Land the wiring** — `buff_frame_wiring::update_buff_frame` at the
   marked seam (f49's world_sim/player.rs:1186-1203 comment; this
   tree's live hook :1230), after `update_summon_buff_arms`, BEFORE
   the lifted `update_survival` (their :1204). The C# order: scan
   :25566 → equips :25633 → armor sets :25685 → regen :25977/:25995.
2. **Apply the 3-ASSIGN bridge** (seal item 2's stub form) in
   `fold_orb` — SAME commit as the wiring landing. Bridge-without-
   writer strands the regen at 0; writer-without-bridge strands it
   too (the scan-local twins have no consumer on f49's tree).
3. **Resolve `world_sim/items.rs` to f49's version WHOLESALE** (their
   R3 step 5 / RT1 item 3): the IsAPickup gate removal + the
   nebula/4143 consume arms + the 12/5 pull; the d79 three-way (their
   RT2) adds the pool-facade rewire (`play_grab_sound` single-slot
   facade) + the skip-gate note. This tree's items.rs is untouched vs
   main — the wholesale take is conflict-free. The 94-drink grant
   resolves to ONE line (`potions.rs:216` shape = their
   `apply_life_and_or_mana` AddBuff(94, 300)).
4. **Keep the extra_regen SUM** (this tree player.rs:11492) over
   f49's accessory-only read (:11345) — their RT1 item 4 rules it.
5. **d115 drops** (tip 7c4fcb755, unchanged): DELETE the inline 321
   arm (their player.rs :6632-6648 — the wiring's booster 321 arm is
   canonical, `w4_f12_brain_buff_crits_and_minion_damage` pins it).
   KEEP their `on_hit_dodge_grant` AddBuff(59, 1800) (:4289-4293) +
   the :25705-25708 orphan-59 clear + the `add_buff_from_net_pvp`
   buffs.rs shape (:200-228) wholesale.
6. **f50 drops + lift + flag fix** (seal item 3's table, tip
   51eefdf84): the inline-63 row, the whole rederive fn + call, the
   doc rewords, the :7607 machines lift to the post-wiring seam
   (BEFORE the regen read), the :6352-6354 set-flag clears moved
   post-scan, the OrbContext live-flag stub replacement, the two
   witness retargets. KEEP the hurt head + machines + dev-3 orbits.
7. **f48 drops** (seal item 3's table, tip e5b66bb06): the panic
   grants/field/spawn/equip-halves/clear + the inline-63 row; the
   dodge block, long_invince consumers and the DD2 205 grant stay.
8. **Post-merge verify**: `python tools/w4_f12_closeout_sweep.py`
   must print `set-diff EMPTY` + the anchors canary
   (`anchors.txt:900` cites the g27 home — their RT1's ":904" is
   stale; :906 the wiring). The
   merged ownership census: 63 DEBUFF-only, 95-100/170-172 g27-only,
   173-181 `nebula_buffs`-only (+ the bridge), 321 booster-only, 59
   grant d115-only, 205 hurt grant f50-only (the DD2 3600 site
   separate), 94 drink one line — every one of the 270 ids exactly
   one live grant path.

## Seal item 5 — research FINAL: the buff-family bible

**Registry number: 28g** (this file). The buff family's single merge
lookup — every member, its C# home, its canonical port owner, its
ruling. The per-phase evidence stays in the W4-f12 docs (sweep-census,
closeout); the stale monitor notes are superseded by seal items 2/3.

| Family member | C# | Canonical owner (port) | Seal ruling |
| --- | --- | --- | --- |
| The slot engine (`AddBuff`/`DelBuff`/`FindBuffIndex`, the re-grant semantics) | `Player.cs:5250-5480` | `buffs.rs` (main + f12) | refresh-to-max default; ONLY 94 (cap 600) / 383 (cap 43200) additive; fed/pet/flask replace-families; d115's `add_buff_from_net_pvp` shape lands wholesale |
| The UpdateBuffs frame + the 270-id dispatch | `:9855-12069` | **f12** `buff_frame_wiring.rs::update_buff_frame` (blob-anchored, anchors.txt:906) | ONE slot-ordered pass; the ownership sets are the sweep's (62/78/32/24/11/37/4/18/66/4); set-diff EMPTY re-run at seal |
| The 18 orb ids 95-100/170-181 (tier writer) | `:10342-10461` + `:12070-12079` | **g27/f12** `buff_effects_orbs.rs::apply_orb_arm` (blob c6bcb017b, identical at g27 tip 1cd7ac367) | f50's rederive + f49's scan arms + d79's copy all DROP/DELETED; the machines (f50) read the scan's counters post-scan |
| The nebula tier FIELDS + bridge | `:12072` assign-derivation | **f49** `Player.nebula_level_*` + `fold_orb_nebula_bridge` (:3547); f12's `fold_orb` calls it at the merge | the 3-ASSIGN stub form (seal item 2); the pickup hint (`nebula_levelup` :3492) is the only other writer, same-tick post-scan |
| The nebula pickup/consume/12-5 pull | `:57030-57059` + GrabItems | **f49** `world_sim/items.rs` wholesale | the d79 cue rewires onto the pool facade (their RT2); the IsAPickup gate resolves to f49's removed shape |
| The regen ENGINE + the extra_regen feed | `:18949-19560` + `:25977/:25995` | **d117** `regen.rs` (adopted by f49); the LIFTED `update_survival` at the seam | the merge keeps f12's SUM (:11492); the lift repairs the fold visibility |
| Panic 63: the hurt grant | `:38237-38240` | **f50** hurt head (:4901-4903) | f48's copy drops; the necklace equip rows f50's |
| Panic 63: the per-tick reader | `:10130-10133` | **f12** DEBUFF_DISPATCH (`buff_effects.rs:149`) | f50 :7165-7168 + f48 :6829-6834 both DROP |
| Ballista Panic 205: the hurt grant | `:38241-38244` | **f50** (:4904-4906) | f48's copy drops; the DD2 3600 grant is a different site (f48, lands clean); the READER row is inline-owned on every tree (not a dup) |
| Brain buff 321 | `:9940-9947` | **f12/d73** booster arm | d115's inline :6632-6648 drops; their 59-grant family stays |
| The beetle/solar recharge machines + orbits | `:16304-16581` + PlayerDrawLayers:4007-4031 | **f50** (machines + dev-3 orbit draws) | LIFT post-scan; the set-flag clears move post-scan (:25618-25620) + the OrbContext live flags |
| Mana Sickness 94: the drink grant | `:51938` | **f49/f12** ONE line (`potions.rs:216` shape) | the additive time-cap semantics stay `buffs.rs` |
| The buff HUD (icons/tooltip/time/cancel) | `Main.cs:43202-43404` | **f12** `ui/resources.rs` + `player.rs::try_removing_buff` | 400 static icons landed (seal item 1); the fullscreen-map call site rides the map tasks |
| Ice Barrier 62 (the one frame anim) | `:10462-10485` | **f12/d73** boosters arm + the persist-seeded pair | sim wired; the draw layer is the player-draw remainder |
| The xN icon text | `CachedProjectileCounterBuffTextHandler.cs:12-30` | **f12** `buff_text` + the 28-handler table | projectile counts, never buff stacks |
| The NPC buff twin | `NPC.cs:89910-92192` | `npc_buffs.rs` (main) | out of the player-buff merge; unchanged |

**Seal verdicts:**

* The BUFF lane is SEALED on `task/W4-f12-buff-effects`: the 270-id
  census is EMPTY-diff, the 400 icons landed, the bridge/lift/sum
  contract is mutually acknowledged with f49's SEALED research/85,
  the Panic-63/ladder keep-one covers all three trees, and the owed
  merger-actions are pinned to exact tip ranges.
* The lane's witnesses are WRITTEN NOT RUN (code-only mode — no
  cargo/tests/rustfmt/graphify invoked on this branch).
* Open remainders OUTSIDE the buff family (NOT sealed away): the
  player-draw layers (beetle/solar/nebula orbit draws beyond f50's
  dev-3, the titanium shader), the fullscreen-map buff column
  (`Main.cs:41335`), the NPC buff completion + net sync (report 28's
  seeds), the MP paladin share (f50's d70 task seed).

### SEAL files-touched

| File | Change |
| --- | --- |
| `docs/research/28g-buff-family-seal.md` | this record (the five seal items + the bible) |
| `crates/tr-app/src/assets.rs` | 2 seal witnesses (the icon enumeration + the full-400 pack) |
| `crates/tr-app/src/buff_frame_wiring.rs` | 2 seal witnesses (the 3-tier assign premise + the 63/ladder dispatch-keeper) |
| `crates/tr-app/src/player.rs` | 1 seal witness (the :11492 extra_regen sum) |
| `docs/research/INDEX.md` | the 28g row + the W4-f12-seal row |
| `docs/research/28-buffs-debuffs.md` | the registry pointer to 28g |
| `docs/ANALYSIS.md` | the BUFF SEAL note |
| `docs/PORT_MAP.md` | the seal row |
| `docs/ROADMAP.md` | the S1-E-6 seal note (the lane's row) |
