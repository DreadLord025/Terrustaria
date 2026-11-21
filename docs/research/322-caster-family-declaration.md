# 322 — CASTER-FAMILY declaration: aiStyle-8 ALL CLAIMED + the 693/665 keep-one (W4-g5 lane close)

Status: LANE-CLOSE DECLARATION + merger contract (d37d-E8 input). Author
lane: W4-g5 (`task/W4-g5-npc-style7-caster-doc`, base `a725acb04`).
Companion contracts: research/108 EXTEND/SEAL (the d37c-E plan, tip
`a341daa12` — E2 kinds-lane rule, E7 merge order, E8 per-lane tables),
research/150 (the number registry — **322** per its §8.1 row 15
ASSIGNMENT, d54 tip `a15cb634b`; chain **244 → 322**: the round's first
claim 244 lost to g48's pre-publish win — see §9). The
phases-1-6 record lives in ANALYSIS "The caster aiStyle-8 draw-parity
(W4-g5, code-only)" + its phases-4-6 section; this doc is the RETIRE
round: the dedupe verdicts, the R4 finalization, the machine census and
the two fidelity fixes the retire audit found in g5's own landed rows.

## 1. The declaration

**The aiStyle-8 caster family is FULLY CLAIMED on the pushed union.**
The C# census (`NPC.cs`, the literal `this.aiStyle = 8;` SetDefaults
rows — exactly 13 hits): 24 FireImp (`:9083`), 29 GoblinSorcerer
(`:15690`), 32 DarkCaster (`:9125`), 45 Tim (`:9326`), 172 RuneWizard
(`:11158`), 281/282 RaggedCaster (`:12644`/`:12658`), 283/284 Necromancer
(`:12672`/`:12686`), 285/286 Diabolist (`:12700`/`:12714`), 533
DesertDjinn (`:14978`), 693 LibrarianSkeleton (`:17727`).

| Kind | cs_id | FINAL proto | Machine owner | Status on the union |
|---|---|---|---|---|
| FireImp | 24 | 24 | base → g5 draw-parity | WIRED (`ai_style_8_step`), L1 radius-20 fix landed |
| GoblinSorcerer | 29 | 15 | base → g5 draw-parity | WIRED, the 100-draw arrival bursts + Next(5) ambient |
| DarkCaster | 32 | 32 | base → g5 draw-parity | WIRED, dungeon-wall gate + Next(3) ambient |
| Tim | 45 | 45 | base → g5 draw-parity | WIRED, bursts + Next(5); shoots the folded 665 (§3) |
| RuneWizard | 172 | **827** (was 354) | **g5** phases 4-6 | WIRED: alpha ramp, six 75-spaced triggers, the 129 RuneBlast, Next(255) alpha dust |
| RaggedCaster ×2 | 281/282 | 281/282 | d38 (pushed, unmerged) | WIRED on d38: the 293 LostSoulHostile shot, Next(2) ambient |
| Necromancer ×2 | 283/284 | 283/284 | d38 | WIRED on d38: the 290 ShadowBeamHostile shot + the ±30 jitter pair |
| Diabolist ×2 | 285/286 | 285/286 | d38 | WIRED on d38: the 291 InfernoHostileBolt (ai0/ai1 target payload) |
| DesertDjinn | 533 | 533 | base → g5 draw-parity | WIRED: the curse volley + the 596 bolt + Kill explosion |
| LibrarianSkeleton | 693 | 693 | **g5** phases 4-6 (canonical, §2) | WIRED: localAI clock, book teleport, the 1092 book shot, ai0-scaled light |

**No aiStyle-8 kind remains unwired on the pushed union.** The
`npc_ai_caster.rs` SPEC (S1-D-3-aistyle7, unwired) stays the draw-census
reference; its retirement is the merge-time keep-one below (§5), NOT a
drop-in swap (the d128 exclusion — a swap without the parity commit
would have been a determinism break; g5 phases 1-2 ARE that commit).

### The d33-style102/103 routing note (the brief's question)

The "style-7 caster" slug is the ROADMAP parent row's mislabel (aiStyle
7 = `AI_007_TownEntities`, `NPC.cs:20812-20815`; the caster arm is
aiStyle **8**). Neither 102 nor 103 is a caster style: **102 = the
SandElemental 541** (the only NPC-space user, `NPC.cs:15902`; the arm
`:40147-40542`; the `Projectile.cs:6146`/`:6234`/`:6243` hits are the
539/573/574 projectile rows — different machine) and **103 = the four
Sand Sharks 542-545** (`:15917`/`:15932`/`:15947`/`:15962`; the arm
`:40543-40723`). d33's modules `npc_ai_style102.rs` (`37c0ce77f`) /
`npc_ai_style103.rs` (`ee8b26681`) are landed UNWIRED (the style73
precedent — `mod` seams only, no axis arm, no kinds); the kinds
541-545 are inside d41's claim-only band 491-560 (research/108 R7's
"skip 521/541-545/547" list). **Routing verdict: the caster lane owes
NOTHING for 102/103 — the wiring rides d33's modules + d41's kinds, not
the caster arm.** d33's own ANALYSIS exclusion note ("the aiStyle-8
casters … EXCLUDED — census-confirmed WIRED") predates g5's parity
commit; its "~100+ draws determinism break" caveat is now the LANDED
deliberate one-time stream shift (the d128 plan), not an open blocker.

## 2. The 693 LibrarianSkeleton DEDUPE-FLAG — keep-one for d37d-E8

**The collision**: g5 phase-4/5 (`feb78f10`/`96f07495`, first-add
2026-11-21T09:32:23+02:00) ported 693 in FULL (identity + exhaustive
matches + the AI arms + the 1092 bolt + the light + the draw); d43
batch-12 (`8fc608bc`, 12:33:02) ALSO rowed 693 (defaults + matches +
sheets + bestiary + the dungeon-wall gate). Both branches descend
`a725acb04`; neither saw the other (research/108's sweep recorded d43
as "base, no work" at `bdd59afba`; d43 pushed 13 batches after).

**VERDICT: g5's 693 SURVIVES the merge; d43's 693 rows RETIRE.**
Rationale (the E3 kinds-lane rule inverted by the E8 d42-ownership-flip
precedent — the FULL implementation owns over the defaults-only row):

1. **g5's row is a superset in behaviour**: the localAI clock
   (`:20839-20842`/`:20994-21005`/`:21106-21111`), the book teleport
   (`AI_AttemptToFindTeleportSpotNearBooks` `:19009-19134`), the 1092
   book shot (`AI_FindNearbyBook` `:62308-62364`), the ai0-scaled light
   (`:21301-21308`). d43's arm explicitly SKIPS the book shot ("the
   book-search feed is unplumbed — the kind casts silently", its
   `npc_ai_bat.rs:437` comment) — dropping g5's arm for d43's would
   LOSE the 1092 wiring.
2. **The stat rows are identical** (both verbatim `NPC.cs:17723-17735`:
   18×40, aiStyle 8, dmg 18, def 2, 70 HP, kb 0.6, value 160, NPCHit2/
   NPCDeath2, {20,31} immunity `NPCID.cs:10060-10066`, npcSlots 2) —
   no semantic conflict, the dedupe is mechanical.
3. **d43's ONE unique contribution — the dungeon-wall gate — is
   ABSORBED on this branch** (the retire fix, §4): g5's draft omitted
   the 693 from `dungeon_wall_gated`; d43's row caught it
   (`NPC.cs:18960`). The fix + witness landed here, so nothing of
   d43's 693 work is lost by the retire.

**The exact drop-list (d43's 693-only hunks, for the merger):**

| File (d43 tip `0253bde90`) | Site | Action |
|---|---|---|
| `npc.rs` | `:1117-1120` the enum variant + doc | DROP (g5's `:459-466` survives) |
| `npc.rs` | `:3799` kb / `:4242` def / `:4677` value arms | DROP (g5's identical arms) |
| `npc.rs` | `:6754` `update_kind_ai` → `update_caster` | DROP (g5's group arm `:6147` covers it) |
| `npc.rs` | `:10899` the solo FindFrame case-693 arm | DROP (g5's group arm `:8101` — same rows 0/1/2 + the grounded snap; d43's reads `ai[1]`, g5's the `cast_windup` mirror — behaviour-identical through the `update_caster` fold) |
| `npc.rs` | `:20688` render key / `:26098` ALL_KINDS / `:26505` stat witness | DROP (g5's rows; ALL_KINDS count re-pin: union 272+… minus 1) |
| `npc/data.rs` | `:596` to_u16 / `:989` from_u16 / `:1965` hit / `:2329` death / `:2770` ai_style / `:3367` size / `:3506` cs_id / `:4136` max_hp / `:4563` damage | DROP (g5's identical arms; the exhaustive matches keep ONE arm per fn) |
| `npc_buffs.rs` | `:494` `&[20, 31]` | DROP (g5's `:490` group arm) |
| `bestiary.rs` | `:2231-2237` the ENTRIES row | DROP (g5's `:642-649` — same credit/net/name) |
| `ui/bestiary.rs` | `:1635` the portrait arm | DROP (g5's `:1600` group arm, same 18×40 box) |
| `render/entities.rs` | `:3429` the draw arm | DROP (g5's `:3188` — same NPC_693 sheet + anchor) |
| `app/netglue.rs` | `:3588` the `spawn_cave_walker` group membership | DROP — **and g5's `spawn_hover` ctor is the keeper**: the 693 is a grounded caster like the 32 (`spawn_hover` group `:3419-3421` carries Tim/DarkCaster/FireImp); d43's cave-walker seed probes a walk column the caster never uses. Same verdict shape as the d38 casters' hover ctor (`netglue.rs:3511-3516` on its tip). |
| `npc_ai_bat.rs` | `:80` the `dungeon_wall_gated` 693 extension | KEEP-ONE onto g5's arm — **already absorbed** (§4); the merger takes g5's line, drops d43's edit of the same fn |
| `npc_ai_bat.rs` | `:437-463` the "book shot is a documented skip" comment + the shoot arm | DROP (g5's live 1092 arm replaces it) |
| `assets.rs` | the NPC_693 sheet-list row | KEEP ONE (identical string; g5's `:530` + BESTIARY_NPC_SHEET_IDS 693 slot — the union re-pins the count once) |

**The 281-286 boundary**: d38's six kinds are NOT in this dedupe (no
overlap — g5 never rowed them). The `dungeon_wall_gated` fn is the one
three-way edit site (base 32 / d38 +281-286 / g5+d43 +693): the merged
match is the UNION `{DarkCaster, RaggedCaster, RaggedCasterOpenCoat,
Necromancer, NecromancerArmored, DiabolistRed, DiabolistWhite,
LibrarianSkeleton}` — the C# group verbatim (`:18960`). The
`ai_style_8_step` shoot block merges as: d38's 281-286 branch FIRST
(the C# order — its branch sits before the `type != 172` Item8), then
g5's Item8-gated 29/45/32/172/693 arms. The ambient tail unions d38's
Next(2) gates (`:21331`/`:21340`/`:21350`) with g5's census — **d38's
tip still SKIPS those gates wholesale** (its header's "skipped
wholesale" note): the merger must extend g5's parity policy A to the
six kinds (one Next(2) per tick each, `:21329-21357`) or record the
residual stream gap explicitly. That is the one open parity item on
the union (owed by the merge wave, not by this lane).

## 3. R4 FINAL — the 665 ChaosBallTim fold-into-ChaosBall STANDS

The phases-1-3 decision (ANALYSIS "Decisions recorded") is re-derived
against the FULL C# surface and **finalized: the fold stays; d43's
distinct `NpcKind::ChaosBallTim` retires at merge.**

The evidence, surface by surface:

| Surface | C# site | 30 vs 665 | Fold verdict |
|---|---|---|---|
| SetDefaults | `NPC.cs:15699-15713` / `:17403-17417` | BYTE-IDENTICAL stats: 16×16, aiStyle 9, dmg 20, def 0, 1 HP, NPCHit3/NPCDeath3, noGravity, noTileCollide, alpha 100, kb 0 | one kind exact |
| The AI arm | `:21519` (`type == 30 \|\| type == 665`) | one shared branch: alpha 255 + the 2×27-dust loop; the launch `num120 = 6` default serves both (`:21410`, no 665 override) | one kind exact |
| Debuff immunity | `NPCID.cs:6251-6257` / `:6258-6264` | both `{31}` | one row exact (the retire fix landed it — the base had `&[]`, §4) |
| Expert scaling / ProjectileNPC / ShimmerImmunity | `NPCID.cs:10447`/`:10448`/`:10467` | 30 and 665 in all three sets together | one kind exact |
| Display name | `NPCID.cs:12195` + en-US NPCs.json | both "Chaos Ball" | one name exact |
| Texture | `AssetInitializer.cs:127-129` (`Images/NPC_{id}`) | NPC_665.xnb ≡ NPC_30.xnb (SHA256-equal, 288 B) | one sheet pixel-exact |
| Bestiary | `NPCID.cs:1443-1449`/`:1450-1456` | BOTH `Hide = true` — `GetExclusions` (`BestiaryDatabaseNPCsPopulator.cs:870-886`) drops both from the automated populator; `FindEntryByNPCID(665)` (`:951`) hits the TRASH entry (`BestiaryDatabase.cs:52-60`), the Underground tag goes nowhere | no vanilla card for either; the port's pre-existing ChaosBall ENTRIES row (base) is a recorded deviation, kept as-is — NOT extended to 665 |
| Kill credit | `NPC.cs:95139-95141` + `ContentSamples.cs:863-873` | the credits DIFFER ("ChaosBall" vs "ChaosBallTim" — `ModifyNPCIds` has no 665 fold) but both land in hidden entries: invisible, and the port's kill drain (`world_sim/npcs.rs:5230`) keys the ENTRIES row, which only the folded kind has | unreachable delta in the port |
| **Banner** | `BannerSystem.cs:505-507` (29/30 → 39) vs `:864-866` (45/665 → **177**) | THE ONE REAL SPLIT: Tim's ball credits TIM's banner, the Sorcerer's ball credits the Sorcerer's | **unreachable in the port**: the banner drain (`world_sim/npcs.rs:5257`) reads `entry.net_id` — the folded kind's entry is net 30 → banner 39 for both. No 665-keyed drain exists (no ENTRIES row, no kind). The C# split is real but rides the hidden-entry credit the port deliberately does not model. Recorded here as the fold's ONE deviation; if the hidden-credit surface ever lands (the bestiary Hide-fold wave), the 665 banner arm (`tr_core::bestiary::npc_to_banner` already carries `665 => 177`, `:668`) re-arms with it |

**Why not d43's distinct kind?** It adds a protocol id, an ENTRIES row
(the vanilla has NONE for 665 — d43's card is one EXTRA vs C#, the
d38-328-PumpkingBlade overclaim shape), a portrait, a bestiary sheet
slot and exhaustive-match arms — for zero reachable behavioural delta.
The E3 rule keeps the implementation with the smaller C#-deviation
surface: the fold.

**The exact drop-list (d43's 665-only hunks, batch-7 `4a0322909`):**
the `ChaosBallTim` enum variant + every exhaustive-match arm (kb/def/
value/alpha-100/no_gravity/hit/death/ai_style/size/cs_id/render-key/
update_kind_ai `unreachable!`/find_frame frame-0/ALL_KINDS/stat
witness), the `bestiary.rs:2415` ENTRIES row (the EXTRA card — drop,
do not keep-one), the `ui/bestiary.rs` portrait arm, the
`npc_buffs.rs:517` `&[31]` (g5's sphere group row now carries it), the
`npc_ai_style9.rs:284` `|| ChaosBallTim` alpha-write extension (the
folded kind already writes it), the `npc_ai_bat.rs:461-463` Tim-shot
rewire (g5's folded `CasterShot` is the keeper), the netglue hover-seed
membership, the NPC_665 sheet row (`assets.rs:1308`/`:2821` on d43's
tip — **NPC_665.xnb is SHA256-identical to NPC_30.xnb**
(`63A36662A4CC88805AEE94BF08DEAD55BD35941FCE1C6765134C5FEDAE6A3480`,
288 bytes each, the retail content itself folds the clone), so the
folded ChaosBall draw is pixel-exact for both net ids; dropping the
duplicate sheet row loses nothing). The
`lang_tables.rs`/`projectile_id.rs`/
`creative_sacrifices.rs` name rows are BASE rows (not d43's) — untouched.

## 4. The retire-audit fixes (landed on THIS branch, code-only)

The retire round re-read g5's own phases-4-6 rows against the C# and
found two defects — fixed here, witnesses WRITTEN NOT RUN:

1. **The 693 dungeon-wall gate** (`NPC.cs:18960-18966`): the C# group
   is `(type == 32 || (type >= 281 && type <= 286) || type == 693)`;
   g5's `dungeon_wall_gated` matched only `DarkCaster` — the 693's
   fallback teleport accepted wall-less spots. d43 batch-12's row
   carried the gate (its commit message cites `:18960`); the keep-one
   absorbs it: the match now `{DarkCaster | LibrarianSkeleton}` (the
   281-286 arms ride d38's merge). Witness
   `w4_g5_the_librarian_skeleton_needs_a_dungeon_wall_or_a_live_skeletron`
   (the wall-less rejection + the AnyNPCs(35) opening, the DarkCaster
   test's shape). **Stream-neutral**: the gate changes acceptance, not
   draw count — the parity census is untouched.
2. **The caster/sphere debuff-immunity census** (`npc_buffs.rs`): the
   base comment claimed "no debuff-immunity rows in the data" — WRONG.
   `NPCID.Sets.DebuffImmunitySets` carries 29 {31} (`NPCID.cs:6244`),
   45 {20,31} (`:6364`), 32 {20,31} (`:6272`), 24 {24,31,323} (`:6227`),
   533 the 14-row set (`:9085-9095`), 25/30/33 {31} (`:6234`/`:6251`/
   `:6279`), 665 {31} (`:6258`). All landed verbatim (the 665 row rides
   the folded ChaosBall kind); witness
   `w4_g5_the_caster_family_immunity_rows_match_the_id_sets`. This also
   repairs the R4 fold's one stat gap (the folded ball now Confused-
   immune like BOTH its C# net ids).

Plus the **E8-style renumber EXECUTED on this branch** (§5).

## 5. The RuneWizard protocol renumber 354 → 827 (the E8 work order, self-executed)

g5 phase-4 picked port-local **354** for the RuneWizard (the vanilla
172 is DD2DarkMageT3's port-local slot, `npc/data.rs:300`). The retire
scan against research/108's CANONICAL plan finds 354 is **d37's
CochinealBeetle** (`750d8e44`, the corridor anchor 354-399, E2 rule 1:
"the kinds-lane owns the corridor"; d37's zero-move receipt
`a341daa12` keeps it). g5 is a wiring lane — its kind MOVES.

**New id: 827.** Derivation: above d39's pushed 817-823 band
(`907234441`) and its R7 re-pin ceiling 826; the scripted scan of ALL
`task/W4-*` tips (`git grep -E "NpcKind::\w+ => 8(2[4-9]|30),"` over
every tip's `npc/data.rs`) returns ZERO claims — 824-830 free
everywhere; 827 sits clear of d36's R7 band 767-816 (its pushed rows
767-771/781/782/793), d120's towns 763-766 and g8's 745-761. The
vanilla `NPC.type` 172 rides `cs_id`/`net_id` (the bestiary row,
`lang_tables`, the NPC_172 sheet, the statue/spawner surfaces) —
**unchanged**; only the internal protocol id + its render-key mirror
move (the S1-F-1 renumber precedent; save-compat: the branch never
shipped, no persisted 354-keyed RuneWizard exists on main).

Executed sites (this branch): `npc/data.rs` to_u16 `:596` + from_u16
`:929`, `npc.rs` render_key `:18656` + the kind doc + the
`w4_g5_the_two_casters_setdefaults_rows` witness tuple. The Librarian-
Skeleton's 693 needs no move (vanilla-free, d43's band overlaps it but
§2 retires d43's row).

## 6. The style-7 (aiStyle-8) caster-machine census — FINAL

The live machine is ONE arm: `npc.rs::update_caster` →
`npc_ai_bat::ai_style_8_step` (the S1-D-3s delegate; the d128 plan's
keeper). Per-kind wiring state on the pushed union:

| Machine half | C# | g5 tip | d38 tip | union verdict |
|---|---|---|---|---|
| stand + 500-init + target | `:20817-20843` | all 7 | all 6 | COMPLETE |
| 172 alpha ramp | `:20844-20854` | YES | — | COMPLETE |
| deferred teleport + bursts | `:20855-20992` | YES (100-draw parity) | YES (draws skipped) | merge takes g5's parity arm |
| 693 localAI clock | `:20994-21005` | YES | — | COMPLETE |
| trigger arms (172 six×75 / 533 wrap / 283-284 / 281-282) | `:21011-21068` | 172+533 | 281-286 | COMPLETE on union |
| 650 teleport (book-first for 693) | `:21083-21112` | YES + the gate fix (§4) | YES (no 693) | COMPLETE |
| Djinn curse volley | `:21117-21162` | YES (the 596 live) | — | COMPLETE |
| 281-286 skeleton shot (290/291/293) | `:21177-21227` | — | YES | **residual: the 290/291 `proj_data` rows are MISSING on d38's tip** (only 293 LOST_SOUL_HOSTILE rowed, `data.rs:2802`; `spawn_hostile` → `spawn_with_data_owned` returns None without a row — the Necromancer/Diabolist shots silently no-op). The 290/291 SetDefaults + AI arms are the merge wave's owed rows (d38's claim, NOT g5's — recorded here so the census is honest) |
| Item8 + 29/45/32 sphere shots | `:21230-21247` | YES (665 folded, §3) | YES | COMPLETE |
| 693 book shot (1092) | `:21248-21263` | YES | skip | COMPLETE (g5 canonical) |
| 172 shadow beam (129) | `:21264-21274` | YES | — | COMPLETE |
| ambient tail (draws + lights) | `:21279-21374` | 7 kinds parity | 6 kinds SKIPPED | union owes d38's Next(2)×3 gates (§2 boundary note) |
| FindFrame pose machine | `:77019-77054` | group arm + grounded snap | group arm | COMPLETE (keep g5's) |
| lights (`npc_lights.rs`) | `:21301-21308`/`:21358-21366` | 693 ai0-scaled + 533 + 24 | — | COMPLETE for g5's kinds |

**Unwired caster kinds remaining: NONE.** The residuals are merge-wave
items (the d38 290/291 rows + the d38 ambient-gate parity extension +
the three-way `dungeon_wall_gated` union), all owed by d38/the merger,
itemized above — no new caster kind or machine is unclaimed.

## 7. The merger checklist (E7 order input)

E7: d34 → d38 → d37 → d40 → d42 → d35-last → small lanes (g5 among
them). At g5's merge:

1. The 693 dedupe: drop d43's hunks per §2's table (d43 merges in the
   kinds-lane slot; if d43 lands FIRST, g5's splice drops ITS OWN
   duplicate identity arms instead and keeps the AI/bolt/light arms —
   the keeper set is the same either way: g5's machine + the absorbed
   gate).
2. The 665 dedupe: drop d43's ChaosBallTim kind + card per §3 (the
   fold stands; the sphere {31} immunity row already covers it).
3. The `dungeon_wall_gated` three-way union (§2 boundary).
4. The `ai_style_8_step` shoot-block splice: d38's 281-286 branch
   before g5's Item8 block (the C# order); the ambient tail unions
   d38's Next(2) gates under policy A.
5. The RuneWizard rides 827 (§5) — no rival; the bijection gate
   (`decodable == ALL_KINDS.len()`) re-pins once after the dedupes.
6. `npc_ai_caster.rs` (the unwired SPEC) retires AFTER the union
   verifies (the d128 plan's last step) — its 13 witnesses' census is
   superseded by the live arm's parity witnesses; drop the module +
   its `mod` seam in the merge wave, not before (the reference is the
   parity audit's cross-check until then).
7. The bestiary/sheet count pins re-derive after the two dedupes
   (g5's +2 entries / +2 sheets stand; d43's 693/665 duplicates drop;
   the 665 card drop is a −1 vs d43's tip, NOT vs main).

## 8. Verification record

- No cargo/rustc/rustfmt/tests/graphify invoked (code-only mandate).
  The three new/changed witnesses (`npc_ai_bat.rs` the 693 gate,
  `npc_buffs.rs` the immunity census, `npc.rs` the renumbered protocol
  tuple) are WRITTEN, NOT RUN.
- C# re-reads in ≤100-line windows: `NPC.cs:15699-15715`/`:17398-17419`
  (the R4 stat proof), `:18950-18974` (the gate group),
  `:19009-19053` (the book search), `:21083-21112` (the 650 block),
  `:21230-21243` (the Item8/665 spawn), `:21279-21376` (the ambient
  census), `:21430-21578` (the aiStyle-9 arm), `:7644-7663`/
  `:78895-78914` (BannerID + the kill-credit gate), `:95139-95141`
  (GetBestiaryCreditId), `:11154-11167`/`:17723-17738` (the two
  SetDefaults), `:15885-15969` (the 102/103 rows), `:40147-40158`/
  `:40543-40554` (the 102/103 arms); `NPCID.cs:1400-1471` (the Hide
  rows 25/30/665/33), `:6225-6294`/`:9084-9105`/`:10060-10066` (the
  immunity rows), `:10447-10467` (the expert/projectile/shimmer sets),
  `:12007`/`:12195` (the names); `BannerSystem.cs:500-513`/`:863-867`
  (the 39/177 split); `BestiaryDatabaseNPCsPopulator.cs:529-546`/
  `:840-894`/`:925-964` (the populator + exclusions + the trash-entry
  tag); `BestiaryDatabase.cs:40-74` (FindEntryByNPCID's trash fold);
  `ContentSamples.cs:850-879`/`:960-1088` (the credit dictionary — no
  665 fold); `AssetInitializer.cs:127-129` (the NPC_{id} texture loop)
  + the retail `NPC_665.xnb`/`NPC_30.xnb` SHA256 compare (identical).
- Branch probes (read-only): d43 tip `0253bde90` (the 693/665 hunks
  enumerated per-file), d38 tip `9c0e3bc54` (the gate union + the
  290/291 row gap), d33 tip `b08acf463` (the 102/103 modules unwired),
  d37 tip `a341daa12` (the 354 corridor + E8), d36 tip `8c5839b03` /
  d39 tip `907234441` (the 767-816/817-823 bands).
- The 827-free scan: every `origin/task/W4-*` tip's `npc/data.rs`
  grepped for `=> 824..830` protocol arms — zero hits.
- The research-number claim: the first scan (pre-commit) enumerated
  every tip's `docs/research/` and saw max = 243 (g58); 244 was taken
  on that stale snapshot. The seal-verification re-scan (§9) finds 244
  adjudicated to g48 by registry-150 §8 — this doc is RENAMED
  **244 → 322** per the §8.1 row-15 assignment.

## 9. Seal verification + the 244 → 322 renumber (the second agent's audit)

A second W4-g5 executor re-verified this round against the C# and the
pushed tips (code-only, ≤100-line windows, direct paths). Verdicts:

**The retire content VERIFIES.** Re-derived independently:
* The R4 stat proof (`NPC.cs:15699-15713` vs `:17403-17417` —
  byte-identical rows), the shared aiStyle-9 arm (`:21519`
  `type == 30 || type == 665`), the {31} immunity pair (`NPCID.cs:6251-
  6264`), the Hide=true exclusions (`NPCID.cs:1443-1456`) and the
  trash-entry fold (`BestiaryDatabase.cs:52-60` — `FindEntryByNPCID(665)`
  at populator `:951` hits `_trashEntry`, the Underground tag goes
  nowhere), the banner split (`BannerSystem.cs:505-507` 29/30→39 vs
  `:864-866` 45/665→177; the port's `npc_to_banner` already carries
  `665 => 177`, `tr-core/src/bestiary.rs:668`).
* **The retail texture fold is SHA256-proven**: `NPC_665.xnb` and
  `NPC_30.xnb` both hash
  `63A36662A4CC88805AEE94BF08DEAD55BD35941FCE1C6765134C5FEDAE6A3480`
  (288 bytes each — `Get-FileHash` on the worktree's junctioned content).
* The 693 gate fix matches `NPC.cs:18960` (`type == 32 || (type >= 281
  && type <= 286) || type == 693`); the immunity census rows match
  `NPCID.cs:6227-6233` (24), `:6244-6250` (29), `:6272-6278` (32),
  `:6364-6368` (45), `:9085-9095` (533's 14), `:7119-7125` (172),
  `:10060-10066` (693), `:6234-6240`/`:6251-6257`/`:6279-6285` (the
  spheres).
* The 827 renumber: d37's pushed tip carries `NpcKind::CochinealBeetle
  => 354` (`npc/data.rs:597`) — the squat is real; a fresh all-tip grep
  for `=> 827,` in `npc/data.rs` returns ZERO claims; d39's pushed band
  is 817-823 (`npc/data.rs:595-601`).
* The d43 drop-lists re-probed at tip `0253bde90` + its retire commit:
  every §2/§3 site confirmed (the 693 identity arms, the solo FindFrame
  `:10899-10918`, the cave-walker netglue seed `:3588`, the 665
  alpha-100 ctor row `:3390`, the style9 `|| ChaosBallTim` extension
  `:284`, the Tim-shot rewire `npc_ai_bat.rs:461-463`).
* **d43's own retire doc (`244-d43-kinds-631-697-retire.md`, pushed
  15:55:04) AGREES on 693** (its §1 table: "KEEP g5 — the wired
  caster"; its §2a drop-list matches §2 here). Its §1 "NOT a dup — d43
  keeps 665" paragraph is WRONG on the bestiary claim: the populator's
  `FindEntryByNPCID(665).Info.AddRange` (`:951`) writes to the TRASH
  entry (665 is Hide-excluded, `GetExclusions` `:870-886`), NOT an own
  card — the R4 fold verdict here stands; the merger adjudicates the
  two docs' 665 paragraphs with this receipt (the C# reads above).

**The number 244 COLLIDED — renamed 322 per the registry's ASSIGNMENT
(not a self-pick).** The first claim scan ran on a stale fetch (max
243). The seal re-scan (fresh `git fetch --prune`, 361 `task/W4-*`
tips, 21 285 `ls-tree` rows) finds key 244 claimed by FIVE docs;
first-add committer dates (`git log --diff-filter=A`):

| Lane | Doc | First add (%cI) | Commit |
|---|---|---|---|
| g48 | `244-c28-18-send-switch.md` | 2026-11-21T14:30:16 | `2b61530b4` |
| d53 | `244-proj-dust-retire.md` (chain 240→244) | 14:56:57 | `a106d13e8` |
| g86 | `244-cursor-item-icon-seed.md` | 15:05:34 | `a5f6650d3` |
| d43 | `244-d43-kinds-631-697-retire.md` | 15:55:04 | `6fbe06250` |
| g5 (this doc) | `244-caster-family-declaration.md` | 16:17:57 | `f044bd315` |

**The registry is the authority, NOT a tips-only sweep**: research/150
at the d54 tip `a15cb634b` (round-4 §8 APPEND-ONLY, 17:11:40 — the
"READ THIS FIRST §8 IS AUTHORITY" banner) adjudicates **244 = g48
WINS** (its pre-publish executed push `2b61530b4` 14:30:16 beats the
registry's own 14:47:50 publication — the §7.1/§7.4 convention; d53's
executed 240→244 rename is CONTESTED, re-issued 275). §8.1 row 15
ASSIGNS this doc **322** (`244-caster-family-declaration.md` → 322,
add `f044bd315` 16:17:57); d43's sibling retire doc takes 321 (row 14).
The §8 block 308-333 is RESERVED — **NO SELF-PICKS**; the second
agent's interim 251 self-pick (a tips-only first-free sweep) was
INVALID under §8 and is corrected here to the assigned 322. (The
older `9dddb32b4` §8.2 draft table said g5→323; the LATEST tip's §8.1
row 15 + §8.2 key-verdict row both say **322** — g65 rides 323, row 16.
The latest append wins.) This doc is renamed 244 → 322; the refs
updated on this branch: ANALYSIS (the retire section), PORT_MAP +
ROADMAP row tails, INDEX, anchors.txt (both rows), and the six Rust
comment citations (`npc.rs` ×3, `npc/data.rs`, `npc_ai_bat.rs`,
`bestiary.rs`). The d43/g86/d53 renames belong to those lanes: d43
already EXECUTED its chain 244→304 (`31310b652`) →343 (`715cf5f91`,
per the §8.7 FROZEN table — its 343 stands, the §8.1 row-14 321 draft
is superseded by the executed §8.7 assignment); d53's 275 re-issue
and g86's 306 are owed on their branches.

**Census addendum — two residuals the §6 table missed** (both
g5-owned, both UNWIRED, recorded not fixed — the parity commit's scope
was the UpdateNPCs arm; these are the strike/spawn surfaces):

1. **The `StrikeNPC_Inner` aiStyle-8 strike-restart block**
   (`NPC.cs:81311-81346`): on a strike, netMode != 1 resets `ai[0]` —
   172 → 450 (the last 75-trigger), 283/284 → `Next(2)==0` → 390,
   533 → `Next(3)!=0` → 181, else → 400 — then `TargetClosest()`;
   the 693 arms `localAI[1] = localAI[0] + 250` and `localAI[2] = 10`
   (the book-circle overlay's wind-down window + the hit-stun seed).
   The port's three strike paths (`npc.rs::hurt`/`hurt_from_projectile`/
   `strike_pooled`) arm `just_hit` but carry NO aiStyle-8 block — the
   172's alpha-reset (`:20851-20853`, the justHit read inside the arm)
   IS ported, but the counter restart + the 693 localAI seeds are not.
   Consequence: a struck caster keeps its pre-strike cycle position
   (the C# re-arms the next cast at 450/400), and the 693's
   `localAI[2]` hit-stun stays 0 (the `:20997-20999` countdown never
   runs; the Extra[289] overlay's `num32` term reads 0). The two rand
   draws (283/284's `Next(2)`, 533's `Next(3)`) are strike-tick draws
   the port's UpdateNPCs census does not cover — they ride
   `StrikeNPC`'s stream position, a separate seam. **Owner: the merge
   wave / a follow-up strike-hook lane; the 533/283/284 draws must be
   consumed at the strike site, NOT in the AI arm.**
2. **The 172/693 natural-spawn rolls**: the RuneWizard's hardmode
   deep-cavern `Next(200)` (`NPC.cs:5009-5013`, ABOVE the Tim roll at
   `:5014-5018` — the port's `spawner.rs:1379-1384` carries the Tim
   roll but not the 172's ahead of it) and the LibrarianSkeleton's
   dungeon book-spawn `Next(8)`-else-`Next(10)` + the
   `AI_FindNearbyBook(closestBook: true, checkPlayerScreenRanges:
   true)` feed (`NPC.cs:2745-2763` — the 694 WaterBoltMimic shares the
   window, d43/c3's kind). Both are spawner-lane residuals (the
   `spawner.rs` header's biome-tables-out-of-scope note); the kinds
   are debug/net-spawnable today. **Owner: the spawner wave; the 172
   roll's insertion shifts the SpawnNPC stream — a coordinated
   draw-parity commit like phases 1-2, not a silent add.**

Neither residual changes the §1 declaration (no unwired caster KIND;
these are unwired SURFACES of wired kinds). The §6 table's "COMPLETE"
verdicts stand for the UpdateNPCs arm itself.