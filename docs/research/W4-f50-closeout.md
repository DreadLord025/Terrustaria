# W4-f50 HURT RETIRE — the five receipts (2026-11-21)

Branch `task/W4-f50-hurt-reactions`, retire phase (after DONE tip
**18eeb4f44**, `origin/task/W4-f50-hurt-reactions` = same at retire read;
`player.rs` blob `156fc3fec`, `projectile_ai_misc.rs` blob `926587ac7`).
Code-only: no cargo/tests/rustfmt invoked; this pass publishes receipts and
docs only — the drop-hunks below are MERGER instructions (the deletions
cannot land on this branch alone: the inline 63 row and
`rederive_beetle_solar_buffs` are this tree's ONLY reader/writer for their
arms until the f12 wiring seam exists here; deleting them pre-merge would
break this branch's own witnesses and strand the ladder — the same
both-halves-one-commit contract as f12's f49 bridge hunk).

All f50 line numbers are from THIS tip's own blobs; cross-branch numbers
from the named tips' own blobs, read via `git show` at retire time.

## Receipt 1 — Panic-63 + rederive-ladder: NOT landed elsewhere → exact drop-hunks re-pinned

**Census (run, this pass):** `git grep -lE
"rederive_beetle_solar_buffs|add_buff\(63, 480\)"` over EVERY
`origin/task/*` branch. Hits: **f12** (`buff_frame_wiring.rs` — the
canonical dispatch + its closeout witness), **f48** (`player.rs` — a THIRD
copy family, see the sub-receipt below) and **f50** (this branch). `origin/main`
carries NEITHER (`git grep add_buff\(63 origin/main` → only the deferred-note
comment at `player.rs:5450`; `beetle_orbs`/`10342` → zero). The f12 ledger
(`W4-f12-closeout.md` RETIRE Receipt 1, tip 22ce2a41f) read f50 at the OLD tip
ff56e6e51; the branch has since landed deviations round 2 (792d8a4c3 +
18eeb4f44), so every f12 range is STALE — re-pinned here at 18eeb4f44.

### DUP #1 — the inline Panic-63 reader (canonical: f12's DEBUFF_DISPATCH 63 arm, `buff_effects.rs:149`, blob e63d47e57 at f12 tip 22ce2a41f)

All in `crates/tr-app/src/player.rs` at 18eeb4f44:

* DELETE **:7165-7168** — the 3-line W4-f50 comment + the
  `63 => self.move_speed += 1.0,` match row (between the `3 =>` Swiftness
  row :7164 and the `104 =>` Mining row :7172). Merged as-is with f12:
  +2 moveSpeed/tick (f12's ruling, unchanged).
* RETARGET **:16199-16215** — witness
  `w4_f50_the_panic_buff_raises_the_move_speed` drives `p.update()`; on the
  merged tree the +1 comes through `update_buff_frame`'s dispatch — needs
  the wiring seam rig, or retire in favour of f12's
  `w4_f12_closeout_panic_63_grants_move_speed_once_through_the_wiring`
  (`buff_frame_wiring.rs:804` at f12 tip).
* KEEP: the hurt-head grant `if self.panic { self.buffs.add_buff(63, 480); }`
  (:4901-4903), the `panic` field + doc (:1486-1493), the spawn default
  (:2856), the equip arms 1578 (:6106-6112) + 1290/6187 (:6116-6123), the
  `ResetEffects` clear (:6735-6737), the three-source grant witness
  (:16117-16151). The grant itself is NOT a dup on f12 (f12's ruling: "their
  hurt-head GRANT stays") — but IS a dup vs f48, see below.

### DUP #2 — the ladder re-derivation (canonical: g27 `apply_orb_arm`, `buff_effects_orbs.rs` blob c6bcb017b — byte-identical at g27 tip 1cd7ac367 and f12 tip 22ce2a41f, re-verified this pass)

All in `crates/tr-app/src/player.rs` at 18eeb4f44:

* DELETE **:4726-4828** — the 7-line doc comment (:4726-4732) + the whole
  `pub fn rederive_beetle_solar_buffs` body (:4733-4828; `hurt_inner`
  follows at :4830). A FULL second copy of the `:10342-10443` slot-ordered
  arms g27 owns (time pins, tier sweeps, the Might `0.1 × orbs` stats, the
  set-gate deletes); two dispatches double-apply the Might stats and
  double-run the delete/rescan walk (f12's ruling, unchanged).
* DELETE **:4642** — the `self.rederive_beetle_solar_buffs();` call inside
  `update_beetle_solar_sets` (:4641-4657); the wrapper keeps ONLY the
  machines: `apply_set_bonus_solar` (:4482-4515),
  `apply_set_bonus_beetle_defense` (:4520-4540),
  `apply_set_bonus_beetle_damage` (:4548-4589),
  `update_armor_sets_always_beetle` (:4666-4724),
  `update_armor_sets_always_solar` (:4625-4629),
  `apply_armor_sound_and_dust_changes` (:4602-4621, deviation 4 — f12's
  machines-only list predates it; it rides the same wrapper).
* REWORD **:4631-4640** (wrapper doc) + **:7599-7606** (call-site comment)
  — the "idempotent per tick" claim is FALSE against the g27 module; the
  new comments cite the machines-only shape + the lift below.
* LIFT **:7607** — `self.update_beetle_solar_sets(rng);` moves out of
  `update_with_grapple` to f12's wiring seam AFTER `update_buff_frame`
  (`world_sim/player.rs:1230` at f12 tip) — C# order UpdateBuffs
  (`Player.cs:25566`) → UpdateArmorSets (`:25685`). The wrapper's
  `&mut dyn DropRng` parameter rides the lift (the orb-orbit jitter pair,
  deviation 3, draws on the `UpdatePlayers` stream either way).
* FLAG-LIFECYCLE FIX **:6347-6354** — the comment (:6347-6351) + the three
  clears `beetle_defense/beetle_offense/set_solar = false` (:6352-6354) sit
  BEFORE the scan in this tree's `update`; C# clears at `Player.Update`
  :25618-25620 AFTER UpdateBuffs. With the machines lifted post-scan, f12's
  `OrbContext::default()` stub (`buff_frame_wiring.rs:155` at f12 tip)
  would read all-false and delete every orb buff. Merge must (a) move the
  three clears post-scan per :25618-25620 AND (b) replace the stub with
  the live `set_solar/beetle_defense/beetle_offense` flags (f12's fix,
  re-pinned).
* RETARGET **:16342-16379** — witness
  `w4_f50_the_beetle_solar_buff_ladder_rederives_the_counts` calls the
  deleted fn 4× (:16352/:16359/:16366/:16373); re-derive against g27
  `apply_orb_arm` (its 22 witnesses already pin these arms) or retire.
* KEEP: the hurt grants 63/205, the solar/beetle hurt cuts (:4976-4978,
  :4986-4987), `consume_solar_flare` (:4456-4473), the 3-lane drains, the
  always-counter zeros, the orb-orbit machine + draw layer, the dust
  request flags, the recharge-machine witnesses (:16287-16340) and the
  orb-orbit witness (:16432+) — none touch the ladder.

### Sub-receipt — the f48 THIRD copy (NEW finding, not in f12's ledger)

f12's sweep censused f50/d115/f49/d73 only. This retire census finds
`task/W4-f48-dodge-2` tip **69d499c6b** (`player.rs` blob f74877839) carries
the SAME panic family, ported independently:

* the hurt-head grants `AddBuff(63, 480)` + `AddBuff(205, 300)`
  (:4514-4531, comment + both ifs) — byte-equivalent semantics to f50's
  :4887-4906 (same values, same post-dodge pre-defense site);
* the `panic` field (:1454-1462) + spawn (:2808) + equip arms 1578/1290/6187
  (:5649-5664) + `ResetEffects` clear (:6391-6392);
* the inline 63 reader row (:6809-6814) — the SAME dup vs f12's
  DEBUFF_DISPATCH arm;
* witnesses `w4_f48_a_dodged_hit_grants_no_panic_and_draws_no_extra_roll`
  (:15769+) and `w4_f48_the_panic_buff_doubles_the_move_speed_until_expiry`
  (:15789-15808).

**Ruling proposed for the merger (f50-canonical hurt head):** W4-f50 owns
`hurt_inner`'s reaction head (the ROADMAP row's scope); keep f50's grants +
field + equip arms + clear, DROP f48's copies (:4514-4531, :1454-1462,
:2808, :5649-5664's `panic` halves — f48's 1578 keeps `honey_comb`, its
6187 keeps `long_invince`, both already f50/main fields — :6391-6392) and
f48's inline 63 row (:6809-6814, f12 canonical). f48's dodge block itself
(the sash/belt/brain/shadow rolls BEFORE the grants) is f48's own lane and
already re-verified on main — untouched. f48's two panic witnesses retarget
onto the surviving grant (the dodged-hit-no-grant assertion composes 1:1
with f50's head — same site after the dodge early-returns). The 205 grant:
f12's tree has no writer (its ruling), so after this fold exactly ONE
`AddBuff(205, 300)` hurt grant survives (f50's :4904-4906); f48's separate
`add_buff(205, 3600)` (:12461, the DD2 crystal lane) is a DIFFERENT site and
lands clean.

## Receipt 2 — f3-stealth keep-one: MUTUAL RECEIPT (compose 1:1 verified at both tips)

f3 tip re-read at retire time: **75f3f7347** ("W4-f3 retire (4): the
PLAYER-MOVE-FAMILY declaration") — `4d820f7d` is an ancestor (verified
`git merge-base --is-ancestor`, exit 0). f3's declaration
(`ANALYSIS.md:39846+` at its tip) names stealth f3-canonical:
`update_stealth` :9856-9976 (the full `Player.cs:25733-25899` chain),
`stealth: 1.0` spawn :3306 (the C# `:1562` default fix), the toggle, the
Always_Vortex disarm, the rings drain; its per-sibling drop table covers
g66/e1 — f50 is NOT in it (f50 ports no machine, only the hurt write).

**f50 → f3 receipt (this branch acknowledges f3-canonical):**

* f50's hurt write `self.stealth = 1.0;` (`player.rs:4907` at 18eeb4f44,
  the `:38245` port) composes 1:1 with f3's write (:4876 at 4d820f7d,
  carried at 75f3f7347): same value, same unconditional post-dodge site,
  no intervening read — verified by reading both blobs this pass. KEEP ONE
  at merge; f3's copy is canonical (its comment cites the machines' re-fade).
* f50 DROP-HUNK for the merger: delete **:4907** + reword the head comment
  **:4887-4900** (drop the stealth-break sentence + the CROSS-NOTE, keep
  the panic/squire grant citations). The invis strip (:4940-4947) is f50's
  own (`:38281-38290`) — f3 never ported it; KEEP.
* f50's spawn default stays `stealth: 0.0` (:3024) on THIS branch — f3 owns
  the 0.0 → 1f fix (:1562); at merge f3's spawn line wins (its declaration
  already lists e1's `stealth: 0.0` spawn as dropped — f50's :3024 joins
  that drop list).
* The shared `stealth`/`shroomite_stealth` field pair: ONE definition, f3's
  (its shared-field list); f50's :2126-2129 doc-comment block dedupes to it
  (f50 adds no field f3 lacks).
* Witness `w4_f50_the_hurt_breaks_stealth_and_strips_invisibility`
  (:16173-16197) stays VALID on the merged tree unchanged — f3's write
  satisfies the same assertions (0.3 → 1.0 on an accepted hit, untouched on
  a refused hit); the invis-strip half is f50's own. KEEP as the joint
  hurt-head witness.

**f3-lane/merger action:** none beyond the existing PLAYER-MOVE-FAMILY
table + this drop-hunk; no f50 machine exists to dedupe. Mutual receipt
published for the f3-lane and the e1b-141 merger: the stealth family is
f3's whole, f50 contributes only the invis strip + the joint witness.

## Receipt 3 — g64-608 keep-one: KEEP-YOURS note (the 608 bodies are f50's)

g64 tip re-read at retire time: **625c449e0** (remainder 4, the e3/g63
keep-one refresh) — `77616a7d` (the wiring wave this branch's deviation-2
claim cited) is an ancestor. **No-608-row CONFIRMED at the new tip:**
`git grep -nE "\b608\b"` over g64's `proj_trail_dust.rs`,
`projectile_ai_misc.rs`, `projectile.rs`, `world_sim/projectiles.rs` → zero
kind-608 hits (every match is a line-number coincidence: `:36064-36084`,
`:12601-12608`, tile-row 608 px). g64's ledger (41 trail kinds + deferred
813/957/670/590) never claimed the Solar Flare blast.

**KEEP-YOURS (f50-canonical 608):** `solar_flare_blast_bodies`
(`projectile_ai_misc.rs:487-593` at 18eeb4f44, the `Projectile.cs:72183-
72224` port: 4× dust 31 + 30× dust-6 pairs + 10× dust 6 + 10× dust 31 + 2×
gore 61-63, every post-arm), the `solar_flare_blast` seam
(`projectile.rs:23416`), the draw-census oracle (:628+, 252 explicit + 578
internal = 830 draws) and the two witnesses stay f50's whole. g64 has no
copy to drop.

**The adapter keep-one (the ONE real overlap):** f50's `BlastRng`
(`projectile_ai_misc.rs:599-626`) vs g64's `DustDraw`
(`proj_trail_dust.rs:59-71`, blob 8fda3f2cd at 625c449e0): the `DustRng`
impl is arm-for-arm IDENTICAL (`next_max → next_range(0, max)` — one
`Sample()` step, `UnifiedRandom.cs:120-141`; `next_range`/`next_float`
delegate). f50's adds the `GoreRng` impl + `next_float_direction`
(`(float)NextDouble() * 2f - 1f`, `Utils.cs:1203-1206`) — g64's `DustDraw`
has no gore half. **Merge ruling:** keep ONE adapter — lift f50's `GoreRng`
impl onto g64's `DustDraw` (the shared seam, g63-shape) and re-point
`solar_flare_blast_bodies`' `BlastRng(rng)` (:499) + the three closure
signatures (:505/:513/:518) at `DustDraw`; delete `BlastRng` (:595-626).
Stream-identical either way (same delegations); the census witnesses pin
the order regardless of the adapter's name.

## Receipt 4 — d70-paladin future-wire: the MP-lane task seed

d70 tip re-read: **a7620eec3**; `ee4905939` is an ancestor (exit 0). The
landed inputs (`net.rs` at a7620eec3): `PlayerNet.team` :580 (fed by 12
PlayerSpawn / 45 TeamChange), `PlayerNet.dead` :585 (the respawnTimer arm +
the hp<=0 stand-in), `PlayerNet::paladin_shield()` :603-612 (938/3997/3998
in functional slots 3..10 above 25% life — re-derived from the synced
inventory, the documented deviation vs the C#'s live peer flag).

**TASK SEED — MP-peer-Hurt-deferred (the paladin team share,
`Player.cs:38338-38368`):** single-player-unreachable FINAL on this branch
(the `m != whoAmI` peer scan finds no second team player; the block
no-ops — documented at the `hurt_inner` seam + ANALYSIS). The MP-lane port
needs, beyond d70's landed fields:

1. the remote players' LIVE `statLife`/`statLifeMax2` (the port's PlayerNet
   carries snapshot hp/max_hp — the C# simulates every player locally;
   the redirect's `Hurt` on the peer must debit real life),
2. a peer `Hurt` path (`Player.Hurt(PlayerDeathReason.ByOther(20), damage,
   0, pvp:false, quiet:false, Crit:false, ImmunityCooldownID.PaladinsShield,
   dodgeable:false)` — `:38361`; the port's `hurt_inner` has no
   cooldown-counter parameter, the PaladinsShield counter is unported),
3. the gate + split verbatim: `defendedByPaladin &&
   TeamDamageShare[cooldownCounter] && num2 >= 4 && Damage < 9999`
   (`:38338`; the set = {General, BossNoCheese},
   `ImmunityCooldownID.cs:51`), the nearest `CanDefendWithPaladinsShield`
   peer scan (`:38342-38354`; the predicate `:38188-38195` = active &&
   !dead && hasPaladinShield && team > 0 && team == otherTeam &&
   life > 25% max), the 0.75/0.25 split (`:38355-38358`), the
   `PaladinsShieldRange` = 800 distance gate (`:38359`, `:57100`),
4. the `PaladinsShieldHit` particle request (`:38363-38366`, the
   ParticleOrchestrator lane — visual, folds until that lane lands).

Already ported and reusable: the `defendedByPaladin` buff-43 flag
(`buff_effects_misc.rs:304/:675`), the buff-43 reader wiring. Depends on:
the MP player-sim (remote equips/life), the cooldown-counter family
(the M5-COMBAT iframe-counter remainder). Owner: the MP lane (no branch
claims it; this seed is the claim's spec).

## Receipt 5 — research/04 SEALED + the hurt-family bible

`docs/research/04-player-combat.md` gains the RETIRE/SEALED section (the
hurt-family bible: every reaction's C# site → port site → owner → merge
ruling, the wave-1 MISSING rows superseded in place by the addendum +
bible). INDEX row 04 flips to ✅ SEALED. The bible is the single lookup
for the hurt family's merge; the ROADMAP W4-f50 row + PORT_MAP row carry
the retire note.

## The ONE-paragraph HURT merger-instruction

**Merge HURT as follows:** take f50 18eeb4f44's `hurt_inner` reaction head
+ cuts + queues + the 3-lane drains as canonical (the panic/squire grants,
the invis strip, the solar/beetle cuts, the football/golf/text/dust
queues), then apply the folds in the SAME merge commit: (1) delete f50's
inline Panic-63 row (`player.rs:7165-7168`) — f12's DEBUFF_DISPATCH 63 arm
is canonical — and the whole `rederive_beetle_solar_buffs` (:4726-4828) +
its call (:4642), keeping ONLY the machines inside
`update_beetle_solar_sets`, LIFT the :7607 call to f12's wiring seam after
`update_buff_frame` (C# :25566 → :25685), move f50's set-flag clears
(:6352-6354) post-scan (C# :25618-25620) and replace f12's
`OrbContext::default()` stub with the live flags; retarget f50's two
witnesses (:16199-16215 panic-speed, :16342-16379 ladder) onto the
wiring/g27 paths; (2) drop f48's panic-family copies (grants :4514-4531,
field :1454-1462, spawn :2808, equip `panic` halves :5649-5664, clear
:6391-6392, inline 63 row :6809-6814) — f50's head is canonical, f48 keeps
its dodge block + the DD2 205 grant; (3) drop f50's `stealth = 1.0` hurt
write (:4907) + its spawn default (:3024) — f3-canonical per its
PLAYER-MOVE-FAMILY declaration — keep f50's invis strip + the joint
witness; (4) keep f50's 608 blast bodies whole (g64 no-row confirmed at
625c449e0), folding `BlastRng` into g64's `DustDraw` + the lifted `GoreRng`
impl; (5) the paladin share stays deferred (the MP task seed, Receipt 4);
then verify with f12's `python tools/w4_f12_closeout_sweep.py` (must print
`set-diff EMPTY`) + the anchors canary + the catalog test command.

## Verification (this pass)

Code-only: no cargo/rustc/tests/rustfmt invoked. Every range re-derived by
direct read of the named blobs (`git show <tip>:<path>` + the working-tree
reads at 18eeb4f44, clean status). The census command over all
`origin/task/*` branches ran to completion (3 hit branches: f12/f48/f50).
The ancestor checks (`4d820f7d` ∈ f3, `ee4905939` ∈ d70, `77616a7d` ∈ g64)
all exit 0. The g27 orbs blob equality (c6bcb017b at g27 1cd7ac367 = f12
22ce2a41f) re-verified by `git rev-parse <tip>:<path>`.