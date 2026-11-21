# 350 — W4-c7 SEAL: the critter-family FINAL census + bible (registry-number adjudicated)

Branch `task/W4-c7-style65-critters` (stacked on `task/W4-c7-style64-fireflies`
tip `c2cc6ec61`, base `a725acb04` wave-3 complete). Seal date 2026-11-21,
fetched `origin/main = 051600da8`. Code-only lane: no
cargo/tests/rustfmt/graphify; witnesses WRITTEN NOT RUN. This doc is the
c7 lane's FINAL artifact — the style-65 extension was CENSUSED, not
re-ported (the canonical machine is d30's, §1); the seal lands the
cross-lane receipts + three witness pins + the family bible.

**Number chain: 283 → 350.** The seal's §0 tips-only sweep self-picked 283
AFTER the §8 publication (`0a658c7b3`, 16:4x) — registry-150's LATEST
(`origin/task/W4-d54-dust-sites-world` `a15cb634b`) §8.7 FROZEN table rules
every post-§8 self-pick contested: c7's 283 row (add `09978b3bd` ~16:5x)
is assigned **350**; 283 itself stays f26's reservation (§8.7 note: "283:
f26-reservation KEEPS; g59→321 (§8.2); c7→350 (§8.7)"). This rename
executes that assignment — NO new self-pick. The §0 sweep below is kept
as the historical record of the contested 283 pick.

## 0. Registry-number adjudication (research/150 rules) — HISTORICAL (283 pick, superseded by §8.7 → 350)

Registry: `150-research-number-registry.md` lives on
`origin/task/W4-d54-dust-sites-world` (tip blob `cf077631f`), §6 refresh:
"Numbers 281+ are FREE … take the next number ≥281 and re-check this
registry AND a fresh all-tip sweep first".

Fresh all-tip sweep (this seal, `git fetch origin --prune` then per-tip
`git ls-tree docs/research/`, **360 tips**): the first sweep found NO doc
≥281; a re-fetch minutes later caught three lanes' pushes — **281 is now
TRIPLE-claimed, 282 DOUBLE-claimed**:

| key | claimant | first-add (`git log --diff-filter=A`) | verdict |
| --- | --- | --- | --- |
| 281 | d30 `281-band-65-72-retire.md` | `ec89521fb` 2026-11-21T15:42:58+02:00 | **WINS 281** (earliest) |
| 281 | d125 `281-empress-sheets-ledger.md` | `5a26ae348` 15:44:38 | loses → owes ≥283 rename |
| 281 | g48 `281-c28-18-send-switch.md` | `9c776ca2f` 15:50:24 | loses → owes ≥283 rename |
| 282 | d125 `282-d125-retire-final.md` | `5a26ae348` 15:44:38 | **WINS 282** |
| 282 | g48 `282-39-item-ownership-seed.md` | `9c776ca2f` 15:50:24 | loses → owes ≥283 rename |

The sweep's max used key was 282, so the seal self-picked **283** as
"first free" — **CONTESTED and SUPERSEDED**: the tips-only sweep missed
§8's reservations (283 is f26's) and the §8.7 FROZEN table assigns this
doc **350** (see the chain banner above). The d125/g48 281/282 losers
were likewise renumbered by the registry's later rounds (§8/§8.1), not
by this seal's ≥283 note. No other lane's file is touched.

## 1. Git-log census — what landed on the c7 lane + the style-65 field

### 1a. The c7 style-64 commits (this branch's base, all pushed)

| commit | phase | what landed |
| --- | --- | --- |
| `81094ca06` | 1 | the four aiStyle-64 kinds + SetDefaults rows: **Firefly 355 / LightningBug 358 / Lavafly 654 / Shimmerfly 677** (`NpcKind` variants, `to_u16`/`from_u16` vanilla ids, `cs_id` hand rows, `ai_style()=64`, size 10x10 (358 12x12), lifeMax 5, dmg/def/value 0, kb class-default 1f, noGravity, NPCHit1/NPCDeath1, bestiary entries, G6 render rows over `FIREFLY_SHEETS`, FindFrame cases 355/358/654 + the 677 pixel-wrap quirk) |
| `045922d95` | 2 | the `64 =>` solo axis arm (`update_rng_style64_solo`), the SIM POOL pass at the slot turn (slot-ordered `Ai64NpcView` threat scan), the `EntityLights` glow drain at the C# call position, `Npc::ai_064_integrate` (dead-band X snap + the 677 raw-vs-flyer-collision split) |
| `c2cc6ec61` | 3 | the two wiring witnesses (`w4_c7_the_firefly_defaults_match_the_csharp_rows`, `w4_c7_the_style64_axis_runs_solo_feeds_the_glow_and_splits_the_integration`) + anchors rows 469-471 + ANALYSIS/PORT_MAP/ROADMAP |

**Brief-correction (census-verified):** the SEAL brief's
"Firefly355/Butterfly356/LightningBug357/etc" mislabels two ids —
**LightningBug is 358** (`NPCID.cs:11393`), **357 is Worm** (aiStyle 66,
c8's kind, `NPCID.cs:11391` region); **Butterfly 356 was NEVER on the c7
lane** (it is the f18/d39 dup, §3). The c7 four are exactly
355/358/654/677.

### 1b. The style-65 machine census (whole `aiStyle = 65` tree scan)

Exactly FOUR NPC rows (`NPC.cs:13535`/`:14279`/`:17254`/`:17355`):

| kind | NPCID | SetDefaults row | box / flags | catchItem | frames |
| --- | --- | --- | --- | --- | --- |
| Butterfly 356 | `NPCID.cs:11389` | `NPC.cs:13531-13544` | 10x10, dmg/def 0, life 5, npcSlots 0.25, noGravity | 1994 (+ai[2]-1 per tick `:45106`, 1994..2001) | 24 (8 colours × 3) |
| GoldButterfly 444 | `:11561` | `:14275-14289` | same zero-stat row, rarity 3 | 2891 | 3 |
| HellButterfly 653 | `:11983` | `:17250-17264` | 10x10, lavaImmune | 4845 (lava bait) | 3 |
| EmpressButterfly 661 | `:11999` | `:17351-17366` | 10x10, lavaImmune, rarity 4 | 4961 | 3 |

(`Main.npcFrameCount` array `Main.cs:67082`, indices 356/444/653/661 =
24/3/3/3, machine-counted from the initializer.) NOT the projectile
aiStyle 65 (Sharknado Bolt 385, `Projectile.cs:29535+`, W-9's
`ai_sharknado_bolt` — a different dispatch).

**The machine is ALREADY PORTED — five copies across the wave, keep-one
adjudicated by d30's SEAL (research/281 §3, tip `ec89521fb`):**

| copy | lane | shape | verdict (d30 281, re-verified here) |
| --- | --- | --- | --- |
| `npc_ai_style65.rs::ai_065_butterflies` `f23b43fde` | **d30** | the WHOLE arm: 661 rainbow-light/fade-despawn head + 653 ember tail + real pool + `Ai65Effects` | **CANONICAL — keep** |
| c10's copy (comment-only divergent, 9+/6-) | c10 | same body + the `npc.rs:5162` 444 adapter | keep the ADAPTER, drop the module copy |
| `ai_065_step` `e098cf888` (07:49:01) | f18 | 356-only subset (no 661 head / 653 tail / pool) | DROP (strict subset) |
| `ai_065_step` `024c9877f` (07:58:34) | d39 | 356-only subset + `in_world2` margin guard | DROP (strict subset) |
| `npc_ai_lacewing.rs` (549 ln) `7a0482302` | d125 | 661-only subset, light/dust deviations | DROP module; KEEP its threat-slice pre-pass + despawn gate + case-661 SpawnBoss drain re-pointed onto `Ai65Effects` |

The c7 lane therefore ports NO style-65 code: the canonical d30 module
+ the roster lanes' kinds (356 f18/d39 keep-one §3, 444 c10, 653/661
d43) compose at merge. The c7 contribution is the style-64 half of the
critter family + this seal's receipts.

## 2. The SIGHT gate — generalized `CountsAsACritter` auto-covers c7

C# property (`NPC.cs:6887-6897`, verbatim):

```csharp
public bool CountsAsACritter
{
    get
    {
        if (this.lifeMax <= 5 && this.damage == 0 && this.type != 594)
        {
            return this.type != 686;
        }
        return false;
    }
}
```

(594 = WindyBalloon `NPCID.cs:11865`, 686 = BoundTownSlimePurple
`:12049` — the two exclusions.) The populator
(`BestiaryDatabaseNPCsPopulator.cs:851-868`) registers every
`CountsAsACritter` NPC with the `CritterUICollectionInfoProvider`
(sight-based, NOT kill-count).

**c8's generalization** (tip `7c2de9c06`, retire phase 4): the port's
nine-kind match became the GENERIC gate
(`bestiary.rs:2280-2285` at the c8 tip):

```rust
fn counts_as_a_critter(kind: NpcKind) -> bool {
    kind.max_hp() <= 5
        && kind.contact_damage() == 0
        && kind.cs_id() != 594
        && kind.cs_id() != 686
}
```

**AUTO-COVER RECEIPT (c7's four, verified at this branch's tip
`c2cc6ec61`):** every c7 kind has `max_hp() => 5`
(`npc/data.rs:3666`) and `contact_damage() => 0` (`:4034`), cs_ids
355/358/654/677 — none is 594/686. All four pass the gate with ZERO
provider wiring. c8's own s7a receipt already verified this at the
pushed tips (c7 `c2cc6ec61`, c9 `2042d77f1`, c10 `5fcc68ae6`, e26
`048cca6cf`, c4 `0df6964be`). The c7-side witness
`w4_c7_seal_the_sight_gate_auto_covers_the_firefly_four` (bestiary.rs,
WRITTEN NOT RUN) pins the predicate against the port's tables + the
549 DD2LanePortal counter-case (passes the raw gate — hp 5
`npc/data.rs:3470`, dmg 0 `:3835` — but is Hide, `NPCID.cs:4910-4915`,
so the populator never registers it; c8's fold row keeps the kill
provider).

**The HIT-side gate is a DIFFERENT set** (do not conflate):
`Player.CanNPCBeHitByPlayerOrPlayerProjectile`
(`Player.cs:20513-20529`) reads `NPCID.Sets.CountsAsCritter` (the
99-id bool set, `NPCID.cs:10460`, machine-counted) gated on `dontHurtCritters` /
minion-sentry shots, with the `IsCritterThatIsHostileToPlayers`
(`NPC.cs:19810-19817`, the `CritterThatCanTurnOnPlayers` × aiStyle-3
arm) and trap overrides. All four c7 ids ARE in that set (355/358/654/
677 appear verbatim in the `:10460` initializer); the port's
`projectile.rs:113-115` documents the block as unported (the Guide to
Critter Companionship item + minion/sentry shot flags are both
unported) — unchanged by this seal, cited for the merger.

## 3. The d37 E8 roster receipts (research/108, tip `fbb948172`)

**c7 — 4 owned, 2 renumber, 0 drop** (E8 table, 108 §E8):

| Kind | old `to_u16`/`from_u16` | new id | cs_id |
| --- | --- | --- | --- |
| Firefly | 355 | **735** | 355 |
| LightningBug | 358 | **736** | 358 |

Keep as pushed: Lavafly@654, Shimmerfly@677. **677 is CANONICAL-c7**:
c9's duplicate Shimmerfly drops to c7 (E8 c9 table: "Shimmerfly: drop
c9@677 -> keep c7@677"); c9's residual round (`2042d77f1`) already
records "c7's identical Shimmerfly dedupes at merge". The c7 pooled
`style == 64` sim arm subsumes c9's kind-gated arm at merge (c9's 109
§4e compose stands). The 355/358 renumber is owed by the c7 lane at
the 108 merge (NOT executed here — this seal is stacked on the pushed
style-64 tip; the renumber hunk touches `to_u16`/`from_u16` + the
witness pins only, cs_id/ENTRIES/sheets/render keys DO NOT MOVE per
R1's seam evidence). c7's three sheet-grid gaps (Firefly 355,
LightningBug 358, Lavafly 654 — FIREFLY_SHEETS packed, no grid slots;
Shimmerfly 677's slot arrives via c9's pushed row, net-covered) ride
the merge-wave gap close (108 S3).

**The Butterfly 356 dup — f18 × d39, keep-one:**

| lane | kind commit | module first-add | protocol id | shape |
| --- | --- | --- | --- | --- |
| f18 | `e098cf888` batch 1 (07:49:01) | `e098cf888` 07:49:01 | **238** | 356-only subset + the FindFrame colour-offset saturation fix (`a832d38ea`: `(ai[2]-1).max(0.0)` — the unreachable ai[2]=0 grid state) |
| d39 | `535775fd1` supersede → `a0d49dc1d` close | `024c9877f` 07:58:34 | **817** (R7 re-pin, was 781) | 356-only subset + `in_world2` margin guard; FindFrame case WITHOUT the saturation fix |

**KEEP-ONE verdict (this seal, per the d37 kinds-lane rule + d30 281
§3):** d39's Butterfly@817 is the ROSTER-canonical row (the 108 R7
band 817-826 is d39's adjudicated corridor; f18's 238 squats the
verify-lane's first-20 port-local band and f18 is a VERIFY lane, not a
kinds lane — its whole 355-359 batch was superseded by the pushed
c7/c8/c9 lanes per 108's d39 row). DROP f18's Butterfly variant/to/
from/ALL_KINDS/ENTRIES rows at merge; KEEP d39's row + wiring, but
COMPOSE f18's two fixes onto it: (a) the FindFrame saturation
(`a832d38ea` — d39's case underflows at ai[2]=0), (b) the `in_world2`
guard is d39's own (keep). BOTH modules drop to d30's canonical
`ai_065_butterflies` (§1b) — the keep-one is about the KIND ROW, not
the machine. The f18 witness file `npc_w4_f18_witnesses.rs`'s Butterfly
row re-points onto the canonical kind at merge.

**653/661 ownership:** d43 rows both (`EmpressButterfly => 661`,
`HellButterfly => 653`, cs==proto, `npc/data.rs:626/:628` at the d43
tip); d125's `PrismaticLacewing => 742` is the 661 ALIAS dup — d30 281
§4 adjudicates the 661 kind-name dedupe (d43's EmpressButterfly name
vs d125's PrismaticLacewing; the d125 module drops, its pre-pass/drain
compose). 444 is c10's roster tail (`GoldButterfly => 444`,
`npc/data.rs:608` at the c10 tip), dup with d40's gold band — c10's
own retire 2/4 (`0487a02af`) already verdicts: d40 keeps the row, c10
keeps the wiring re-keyed on the canonical kind.

## 4. The gold-critter peer table — c8's 13 rows, c7's gold kinds = NONE

C# `NPCID.Sets.GoldCrittersCollection` (`NPCID.cs:10121-10125`):
**14 members** — 443, 442, 592, 593, 444, 601, 445, 446, 605, 447,
627, 613, 448, 539. `Sets.IsGoldCritter` (`:10126`) is the SAME 14 ids
(the `UpdateNPC_CastLights` sparkle arm's family, `NPC.cs:92345-92368`:
the `Next(300) < maxRGB/30` gated dust-43 sparkle).

The provider registrations are **13, NOT 14**
(`BestiaryDatabaseNPCsPopulator.cs:703-716`): 593 GoldGoldfishWalker is
collection-only (its own entry rides the plain critter provider).
c8's `GOLD_PEER_TABLE` (bestiary.rs:2332-2346 at the c8 tip, 13 rows,
verbatim re-verified):

| gold id | peers | lane |
| --- | --- | --- |
| 443 GoldBunny | {46 Bunny} | d40 |
| 442 GoldBird | {74 Bird} | d40 |
| 592 GoldGoldfish | {55 Goldfish} | c4 (row d42@592 per E8 flip) |
| 444 GoldButterfly | {356 Butterfly} | d40 row / c10 wiring / d39 peer (§3) |
| 601 GoldDragonfly | {599 RedDragonfly} | unported (d40 forecast) |
| 445 GoldFrog | {361 Frog} | d40 + e26 (peer) |
| 446 GoldGrasshopper | {377 Grasshopper} | d40 |
| 605 GoldLadyBug | {604 LadyBug} | e26 |
| 447 GoldMouse | {300 Mouse} | d40 |
| 627 GoldSeahorse | {626 Seahorse} | unported |
| 613 GoldWaterStrider | {612 WaterStrider} | unported |
| 448 GoldWorm | {357 Worm} | c8 (ported, live) |
| 539 SquirrelGold | {299 Squirrel, 538 SquirrelRed} | unported — the ONE two-peer row |

**c7's gold kinds: NONE.** None of 355/358/654/677 is in the 14-member
collection or the 13-row table (the firefly family has no gold
variants in vanilla). The c7 lane owes NO gold row and NO sparkle
wiring; the 444 GoldButterfly's {356} peer row activates when the §3
keep-one lands the canonical Butterfly. The c7-side witness
`w4_c7_seal_the_gold_table_has_no_firefly_row` (bestiary.rs, WRITTEN
NOT RUN) pins the absence against the port's ENTRIES + the c8 table's
13-row shape (the table itself lands with c8's merge — the witness
keys on the cs_id set, not the table's presence).

## 5. The critter-family bible (one paragraph, merger-facing)

The critter family is the zero-stat (lifeMax 5 / damage 0 / defense 0 /
value 0, kb class-default 1f, NPCHit1/NPCDeath1) noGravity-or-grounded
catchable band: the **fireflies** (aiStyle 64, `NPC.cs:34019-34248`,
c7's four 355/358/654/677 — the scale latch, the rearm far/near wander,
the 677 margins + threat flee, the per-type glow, the 1/80 blend, the
per-hit down-scan damps; module `npc_ai_style64.rs`, WIRED end-to-end
on this branch), the **butterflies** (aiStyle 65, `AI_065_Butterflies`
`NPC.cs:44896-45119`, the four 356/444/653/661 — the 661 rainbow
fade-despawn head, the colour ladder + scale latch, the 1/60 blend,
the 3-tile down-scan / 30-tile up-scan bounces, the 15/10-tick threat
flee, the 653 ember tail; canonical module d30's
`ai_065_butterflies`, kinds f18/d39-keep-one + c10 + d43), the
**crawlers** (aiStyle 66, c8's nine 357/374/375/448/484-487/606), the
**snails** (aiStyle 67, c9's 359/360/655 + the shimmer Transform(677)
chain), the **waterfowl** (aiStyle 68, c10's eight) and the fish/frog/
ladybug tails (c4/e26). EVERY member auto-covers the bestiary sight
provider through the generic `counts_as_a_critter` gate (§2) — no
per-lane provider wiring; the gold variants ride the 13-row
`GOLD_PEER_TABLE` (§4) once their rows land. The catch chain
(`bug_net.rs`, S1-P-i36) keys on cs_id + the live ai[2] (the 356
colour re-derive `1994 + ai2 - 1`), the lava-bait split
(`is_lava_bait` 4845/4847/4849) and the release colour seam are
already table-complete for every critter id incl. the unported ones.
Merge order: the d37 108 bible (d34→d38→d37→d40→d42→d35-renumber-last
→small-lanes) with the E8 renumbers (c7's 355→735/358→736 owed), the
keep-one drops (§1b modules, §3 kind rows) and the sheet-grid gap
closes; the style-64/65/66/67/68 axis arms compose (c7's pooled 64
subsumes c9's kind-gated arm; d30's canonical 65 subsumes the four
subset copies).

## 6. Witness register (WRITTEN NOT RUN, code-only)

| witness | file | pins |
| --- | --- | --- |
| `w4_c7_seal_the_sight_gate_auto_covers_the_firefly_four` | `bestiary.rs` | the generic predicate over the port's tables for 355/358/654/677 (hp 5 / dmg 0 / cs_id ∉ {594,686}), the 549 Hide counter-case, the entry net_ids |
| `w4_c7_seal_the_gold_table_has_no_firefly_row` | `bestiary.rs` | none of the c7 four is a gold critter (the 14-id collection absence), the 444→{356} peer contract shape |
| `w4_c7_seal_the_roster_e8_receipts_hold` | `npc.rs` | the c7 four's current protocol ids (355/358/654/677 — the E8 renumber is merge-time), the cs_id hand rows, the Shimmerfly canonical-owner pin (677 decodes to c7's kind), the frame-count rows 4/4/4/4 |

No cargo/tests/rustfmt/graphify invoked. The witnesses compile against
the CURRENT tip's tables only (the c8 gate/table land with c8's merge;
the witnesses pin the c7-side facts that hold TODAY).

## 7. Cross-refs

- ANALYSIS "The critter-family SEAL (W4-c7, research/350)" (the
  deviation-free receipt paragraph).
- PORT_MAP: the aiStyle-64 row's seal tail + the new aiStyle-65
  census row (the d30-canonical keep-one).
- ROADMAP: the S1-D-3 aiStyle-64 row's seal addendum.
- anchors.txt: the three witness canary rows.
- research/150 (d54's registry), research/108 (d37's roster bible),
  research/262 (c8's crawler FINAL, renamed 143→238→262),
  research/281 (d30's band 65-72 keep-one bible).
