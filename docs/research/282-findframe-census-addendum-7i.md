# 282 - FindFrame census addendum after f26 slice 7i (was key 68; renumbered per registry-150 §3/§6.3/§7.3, KEEPS §9.1: 7c->201 7e->203 7f->205 7g->251 7h->281 7i->282 7j->283; executed at the plan-B3 merge of task/W4-f26-findframe-extra-3)

> RENUMBER # 68 addendum — FindFrame census after f26 slice 7i (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7i, base tip `89cef0b45` (slice 7h code) + `7901cd6e9` (the finalized
7h docs). **Code-only mode: no cargo/rustc/tests/graphify invoked** —
every claim is git/file-read evidence against the PUSHED remote tips,
re-probed at census time (`git fetch origin --prune`).

The master wire-census lives at `docs/research/200-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1`. Its §6 carries the **130-kind
UNCLAIMED worklist** (the refresh-3 base, unchanged as a worklist).

## The window re-probe (at the CURRENT tips)

The pool: 36 (the 7h addendum post-state). Per-tip re-probe:

* **d44 tip ADVANCED `c863a7a5` → `ce2c3a24a`** (refresh 4): the wire
  round 4 lands the extra7c (our f26 7c claims) + extra8 (f27) arms
  behind the `cs_id` hook — WIRED 166→196, but every landed id was
  already subtracted in our ladder (the f26/f27 claims); §6's
  130-row worklist base stands, and its 646-649 rows still say
  **UNCLAIMED** (`NPC.cs:76859-76862`). No new independent claim.
* **d42 tip ADVANCED `2d7ac08c` → `33b3cfe07`**: the rotation
  extraction (`npc_rotation_allowed`, the 43-kind write-census) +
  the BloodSquid-619 hover fix — no new FindFrame ids, the 38-id
  subtraction stands.
* **d41 tip ADVANCED `5d036721` → `727578f2d`** (batch 3: the desert
  antlions + Dune Splicer 508-512 SetDefaults/AI) — dedupe inside our
  own 7c claims, no new ids.
* **c8 tip ADVANCED `708522fc` → `7c2de9c06`**: the retire phases
  1-4 (research rename 143→238, the receipts, the crawler-family
  declaration) — the 606 arm unchanged, the 8-id inline subtraction
  stands.
* **f27 `1a24a133` / c4 `0df6964b` / e26 `048cca6c` / g10 `5dee2865`**
  UNCHANGED.
* **Mechanical claim grep**: all EIGHT lane tips grep ZERO for
  `GemBunny` and `\b64[6-9] =>` over `npc.rs` / `npc/` /
  `npc_findframe*.rs`; this branch's own tree greps ZERO arms (the
  only kind-name hits are `creative_sacrifices.rs` / `itemdata.rs` /
  `lang_tables.rs` data rows).

## The census count

The 7h ladder (130 − 15 − 9 − 38 − 8 − 11 − 5 − 8 = 36) re-verified
unchanged at the new tips, then:

| step | subtraction | remaining |
| --- | --- | --- |
| the 7h post-state | — | 36 |
| f26 slice-7i THIS claim (646/647/648/649) | 4 | **32 unclaimed** |

The 32-id post-slice worklist: **634** (the RNG SporeBat — its
seam contract is published in the extra7h module header for the
d44f wave), **650-655** (6 — the DEFERRED bunny tail 650/651/652 +
653 HellButterfly / 654 Lavafly / 655 MagmaSnail; 656 g10-claimed),
**657-662** (6), **664** (1), **667** (1), **669** (1),
**671-677** (7 — 670 g10-claimed), **687-689** (3), **691-696** (6)
= 1+6+6+1+1+1+7+3+6 = **32**.

## This slice's claim (4 kinds — the SMALL 400-storm batch)

646/647/648/649 GemBunnyAmethyst/Topaz/Sapphire/Emerald
(`NPCID.cs:11969/:11971/:11973/:11975`) — the hop group
`case 46/47/303/337/443/464/540/614/646-652` (`NPC.cs:76851-76904`,
our labels `:76859-76862`; 540 the f27-claimed twin — the VERBATIM
ONE-group machine copy, the 7g/7h precedent): the grounded ±1 flip
(`:76868-76875`), the idle row-0 zero + break (`:76876-76881`), the
`|vx| * 1f` THEN `++` STRICT >6.0 step (`:76882-76888`), the
`>= count` wrap back to row **0** (`:76889-76892` — the FULL 7 rows
walk; NO count−1 decrement, NO lift), the air row **4** rising
(`:76894-76898`) / row **6** falling (`:76899-76903`) both
counter-zeroed sprite-UNTOUCHED. All seven bunnies aiStyle 7
(`NPC.cs:17238-17248`, `:17242`) — the hop arm reads no aiStyle (no
dead branch, unlike the squirrel's backpedal).

**650/651/652 deliberately DEFERRED** (the brief's 4-kind cap — they
share this exact arm, a straight seam-extension next slice). 634
stays RNG-excluded (`Main.rand.Next(10)` `NPC.cs:74844`).

Zero-RNG + zero-world-read mechanical grep over the cited range
(`:76851-76904`, 54 lines) = ZERO. Frame counts cross-checked
`Main.cs:67082` vs `NPC_FRAME_COUNT`: 646-649 = **7** each
(`:67148` entries 8-11); all 697 slots re-diffed = ZERO mismatches
(a mechanical text-parse diff).

Module `crates/tr-app/src/npc_findframe_extra7i.rs` +
`find_frame_kind_extra7i` seam + `main.rs` registration; 6 module
witnesses WRITTEN NOT RUN (code-only).
