# 341 — NPC.cs dust-site window coverage (W4-e2 SEALED)

> **Renumber chain: 64 → 285 → 341.** registry-150 §8.7 (FROZEN table,
> `9dddb32b4` on task/W4-d54): e2's post-§8 self-picks 285/286 → **341**
> (coverage) + **342** (bible). All refs repointed at the rename.
>
> W4-e2 batch 6 + seal · 2026-11-21 · code-only census. Which `Dust.NewDust*`
> windows in `decompiled/Terraria/NPC.cs` are claimed by which lane, at
> the fetched branch tips. Companion to d54f's `131-dust-master-status.md`
> (the Main/world-lane keep-one register); this doc is the **NPC.cs AI +
> HitEffect dust** slice. Site id = the C# `Dust.NewDust*` line number.
> "Wired" = the emitter rolls the live `DustPool` on the `UpdateNPCs`
> stream; "staged" = the body + RNG order are pinned in a pure emitter,
> the machine wiring lands with a later wave; "plan" = a count/geometry
> seam the dust lane fills.
>
> **Renumber 64 → 285 (registry-150 post-cutoff rule, one race):** key
> 64 belongs to W4-d64 (`64-drawdata-cases-4.md`, first-add `161c63898`
> 11-20T18:54:02 — the registry's §1 winner row); this doc's first push
> (`bfc081305`, 11-21T10:48:45) came AFTER the registry's final sweep
> (`38bfd1d22`, 10:22:14), so per the cutoff note the lane
> self-adjudicates ≥281 after a fresh sweep. The first sweep found 281
> taken EIGHT ways (d9/d30/g48/f39/c25/c9/d106/d125), 282 three ways
> (g48/d125/f39), 283 = f4's executed `283-npc-dust-final.md`
> (`26728bd87`), 298 = g63's family-final; the interim 284/285 pick was
> RACED — the pre-push re-fetch caught g52's executed
> `284-g52c-relay-host-arms-census.md` (`0224fff80`, 16:19:37; an
> executed push beats the unpublished pick, the f4-281 precedent).
> **285 = this coverage doc, 286 = the npc-dust-family bible** (both
> zero-claimant at the final pre-push sweep, 285/286/287 probed) — that
> pick was later SUPERSEDED by registry-150 §8.7: **341/342 FINAL**. The
> old key-64/284/285/286 citations in commit messages stay as history; the one
> code ref (`npc_dust.rs` module banner) is repointed to 341.

## 1. Lane tips (seal re-fetch 2026-11-21; the batch-6 read in brackets)

| Lane | Branch | Tip (seal) | Scope |
| ---- | ------ | ---------- | ----- |
| d52 | task/W4-d52-dust-sites-npc | `231b31555` (unchanged) | first-40 AI lane (Golem/EoC/caster/sphere) |
| e2 | task/W4-e2-dust-sites-npc-2 | `d5ae8725c` [`9912498fc`] | second batch (this doc's lane) |
| f4 | task/W4-f4-dust-npc-3 | `26728bd87` [`bcf917437`] | third batch + wave E gore — **SEALED**, research/283 |
| g61 | task/W4-g61-npc-spawn-dust | `6e922fc60` [`b5ff1803d`] | spawn/teleport-in/transform visuals — SEAL + 238 renumber |
| g62 | task/W4-g62-npc-hit-dust | `32af98b44` (unchanged) | HitEffect every-strike dust |
| g59 | task/W4-g59-npc-death-effects-1 | `d277f782b` [`29c13d103`] | HitEffect death halves — ACTIVATIONS-2 b14-16 |
| g60 | task/W4-g60-npc-death-effects-2 | `30fea0fe1` [`19431e385`] | death-family matrix — research/110 FINAL-FINAL |
| f5 | task/W4-f5-dust-proj-3 | `b68eac297` [`d0619d8e9`] | wave D gore chain (853/853) — GORE SEAL research/111 |
| f10 | task/W4-f10-gore-sites-3 | `012bdcec3` [`05f40324e`] | wave C HitEffect gore table (1441 sites, b170 COMPLETE) |

The seal-round advances are all downstream of the batch-6 rows below:
f4's b7 + SEAL absorbed the e2-b5 multiply order into its keep-one
bodies (research/283 §1) and fixed both storm tile probes to
`(int)x >> 4`; g61's SEAL renumbered its final doc 237→238; g59 added
the 17-kind activation census; g60 published the f10-overrun
DROP-HUNKS; f5 sealed the 4 hit-half gore wiring assignments; f10
closed the gore chain at :89831. None changes a §2 row's ownership.

## 2. AI() dust windows (`NPC.cs:19344-44037` + the named machines)

| C# window | Kind(s) / machine | Owner | Status | Evidence |
| --------- | ----------------- | ----- | ------ | -------- |
| 19344-19714 | Golem body/fists (AI_045/047) | d52 b1 | wired | `e01179d5e` |
| 20064-20379 | Eye of Cthulhu (AI_004) | d52 b2 | wired | `f86731ed6` |
| 20867-20987 | caster teleport bursts (aiStyle 8) | d52 b3 | wired | `7ce6e95c8` |
| 21284-21368 | caster ambient tail (aiStyle 8) | d52 b4 | wired | `e55b04b23` |
| 21524-21562 | sphere family loop (aiStyle 9) | d52 b5 | wired | `eb084fa9a` |
| 22285/22292 | Skeletron blood trail | e2 b1/b2 | wired | `1da18d19`/`2bfaa71a` |
| 24520 | Antlion 69 fall-drip (aiStyle 19) | e2 b6 | staged | `9912498fc` |
| 24850/24866 | Pixie 75 / Ice Elemental 169 (aiStyle 22) | e2 b6 | staged | `9912498fc` |
| 26462/26755/26760 | Retinazer (aiStyle 30) | e2 b1/b2 | wired | `1da18d19`/`2bfaa71a` |
| 27076/27369/27374 | Spazmatism (aiStyle 31) | e2 b1/b2 | wired | `1da18d19`/`2bfaa71a` |
| 29214/29427 | IceTortoise (style 39) | e2 b3 | wired | `98f3337e` |
| 29540-29572 | SolarSroller ball (style 39) | e2 b3 | wired | `98f3337e` |
| 29919-29948 | ChatteringTeethBomb (style 41) | e2 b3 | wired | `98f3337e` |
| 30897/30913/30922 | EyeballFlyingFish 587 (style 44) | c4 | wired | `npc_ai_style44.rs` |
| 31576/31579 | free Golem head 228 (AI_048) | e2 b2 | wired | `2bfaa71a` |
| 32416 | Brain retreat burst (AI_054) | e2 b2 | wired | `2bfaa71a` |
| 32755 | DungeonSpirit 288 (style 56) | smallkinds | wired | `npc_ai_style56.rs` |
| 32823 | Everscream 344 (style 57) | S1-F-13c | wired | `npc_ai_frost_boss_b.rs` |
| 35157/35162 | MartianTurret 387 materialize (style 73) | e2 b6 | staged (plan seam) | `9912498fc` + `npc_ai_style73.rs` |
| 35279-35486 | MartianDrone/SolarCorite (style 74) | c11/d30 | wired | `npc_ai_style74.rs` |
| 36029-36050 | saucer dead-crew dust (style 75) | saucer | wired | `npc_ai_style75.rs` |
| 36741/36750 | Moon Leech Clot rings (AI_082) | e2 b2 | wired | `2bfaa71a` |
| 37289/37307 | ShadowflameApparition 472 (style 86) | e2 b6 | staged | `9912498fc` |
| 37337/37464 | Squidhead 521 228 rings (style 86) | e2 b6 | staged | `9912498fc` |
| 38958 | Flying Dutchman 228 trail | e2 b2 | wired | `2bfaa71a` |
| 39007/39022 | pillar dissolve body/plume | e2 b4 | wired | `1ec45ba2` |
| 39039/39052 | pillar 6-burst + 241 tail | e2 b5p1 | wired | `0f0d7036` |
| 39200-39362 | pillar ambient per-type plumes | e2 b4 | wired | `1ec45ba2` |
| 39421 | Stardust Cell growth ring | e2 b2 | wired | `2bfaa71a` |
| 39524/39559/39571 | NebulaFloater 420 (aiStyle 97) | e2 b5/b6 | wired (faithful b6) | `31c91d90`/`6c424595f` |
| 39963/39974 | SolarGoop 519 (aiStyle 99) | e2 b5 | staged | `31c91d90` |
| 40024/40035 | AncientLight 522 (aiStyle 100) | e2 b5 | staged | `31c91d90` |
| 40100/40110/40120 | **AncientDoom 523** (aiStyle 101) | e2 b5 | staged | `31c91d90` (b6 attribution fix) |
| 40193/40302 | SandElemental 541 (aiStyle 102) | e2 b5 | staged | `31c91d90` |
| 40834/40851/40947 | EterniaCrystal win/loss rings | **e2 b5 + f4 b1** | wired (OVERLAP) | `31c91d90` / `37d143c75` |
| 41336-41403 | MoonLordCore drama storm | **e2 b5 (41367/41396) + f4 b6 (full)** | wired (OVERLAP) | `31c91d90` / `abff45cf8` |
| 42222 | MoonLordHead deathray fan | **e2 b5 + f4 b6** | wired (OVERLAP) | `31c91d90` / `abff45cf8` |
| 42675/42873 | TrueEye orbit pair + deathray fan | **e2 b5 + f4 b6** | wired (OVERLAP) | `31c91d90` / `abff45cf8` |
| 43251/43278/43365 | KingSlime shrink/grow/body trio | **f4 b2 + e2 b6** | wired (OVERLAP) | `23e313f60` / `1cbbcb95d` |
| 44037 | Deerclops damage-ramp 109 trail | f4 b1 | wired | `37d143c75` |
| 49029/49181/49449/49661 | Fishron materialize + charge trails | f4 b1 | wired | `37d143c75` |
| 49801 | Destroyer family fade-in | f4 b3 | wired | `0b7b14e29` |
| 50656-50713 | AI_005 hover dust gate (50702/50710) | f4 b5 | wired | `87da911e4` |
| 51046/51054 | StardustWorm 570th-tick burst | f4 b1 | wired | `37d143c75` |
| 51656/51662 | Crawltipede head twin trail | f4 b4 | wired | `8d5e4e7cf` |
| 52259 | EoW spawn-fade tail | f4 b4 | wired | `8d5e4e7cf` |
| 71938 | FindFrame case-398 open burst | f4 b6 | wired | `abff45cf8` |

### Unclaimed / other-lane AI windows (not e2's slice)

| C# window | Kind(s) | Lane note |
| --------- | ------- | --------- |
| 44937-45879 | Empress of Light (AI_120) | `npc_ai_empress.rs` (empress lane, argument-draw seams) |
| 46044-47437 | HallowBoss / bow-launch | `bow_launch.rs` / projectile lane |
| 50268/50284 | BloodSquid 619 day-flee burst (NOT Wyvern — f4 research/283 §2 correction; AI_005) | kind wave (619 unported; the hover family d22/f4-b5) |
| 50692 | BloodSquid 619 hover unconditional arm (no Next(20) gate) | kind wave (f4-b5's gate models the bounce/else arms only) |
| 50976 | leech 117 tunnel-entry 20×5 burst | **f4 b7 WIRED** (`e7ea8b24b`, the AI_006 seam's one live site) |
| 50993/51015/51026 | CultistDragon 454-459 / BloodEel 621-623 fades | wyvern-lane census (`npc_ai_wyvern.rs`), kinds UNPORTED — dormant, cited (f4 research/283 §2) |
| 51656+ worm tails | worm pass | f4 b4 seam + worm lane |
| 55868-58807 | town / fighter ambient | town lane (c-series) |
| 60349-63970 | misc AI (unicorn, pirate, DD2) | unicorn_b / pirate / DD2 lanes |
| 65503-66432 | cultist / martian | cultist (c1) / martian lanes |
| 67925-69003 | misc | various |
| 74009-74846 | FindFrame dust | findframe lanes (d44-d49) |
| 90070-94940 | UpdateNPC generic / buff VFX | g61 (90242/90258 alt-texture), buff-VFX wave (90977 soul-drain) |

## 3. HitEffect dust windows (`NPC.cs:81660-89834`)

| C# window | Owner | Status | Evidence |
| --------- | ----- | ------ | -------- |
| 81660-89834 every-strike | g62 | wired (29 batches) | `npc_hit_dust.rs`, `471a68e62`..`32af98b44` |
| 81660-89834 death halves (dust-only kinds) | g59 | wired (13 batches) | `npc_death_dust.rs`, `46368d449`..`29c13d103` |
| 81660-89834 death-family matrix | g60 | wired | `19431e385` (trailcache widening) |
| 85560+ classic-mob gore chain | f5 wave D | wired (853/853) | `d0619d8e9` |
| 82206+ HitEffect gore table | f10 wave C | wired (137 batches) | `05f40324e` |
| 81660-89834 gore bodies | d55 / e5 | gore lane | (gore, not dust) |

## 4. Spawn / teleport / transform windows

| C# window | Owner | Status | Evidence |
| --------- | ----- | ------ | -------- |
| 77494-77515 BigMimicSpawnSmoke | g61 | wired | `0a4af1908` |
| 80963-81033 TransformVisuals | g61 | wired | `0a4af1908` |
| 81035-81058 NPC.Teleport | g61 | wired | `6345f7b7b` |
| 92194-92241 UpdateNPC_TeleportVisuals | g61 | wired | `8d23e2985` |
| 90236-90263 alt-texture transitions | g61 | wired | `8d23e2985` |
| Main.TeleportEffect styles 0-13 | g61 | wired | `6345f7b7b`/`8d23e2985` |

## 5. Overlap register (merger-wave keep-one) — SEALED both sides

These windows were ported independently by two lanes (the branch list
is the claim record, but the :42873-44037 window was small enough that
f4's third batch and e2's second batch both reached it). The merger
takes ONE body per site; the witnesses supplement.

**SEAL receipt (mutual):** f4's research/283 §1 (`26728bd87`) confirms
this register verbatim and publishes the e2-side DROP list — the
merger drops e2's inline KingSlime dust arms (`1cbbcb95d`), the
EterniaCrystal trio + the MoonLord storm/fan/eye bodies (`31c91d90` +
`6e304b960`); e2 KEEPS the named `king_slime_teleport_ring`/
`king_slime_body_drip` emitters + all machine/emitter witnesses as
supplements, re-mirrored onto f4's bodies. f4's b7 absorbed the e2-b5
left-to-right f32 multiply order into its keep-one bodies and fixed
BOTH storm tile probes to the C# `(int)x >> 4` (`Utils.cs:1807-1810`).
The e2-side seal fix `d5ae8725c` closes the last divergence this side:
the head/eye fan ANGLE now truncates the NextDouble to f32 BEFORE the
2π multiply (`:42221`/`:42872`'s `(float)NextDouble() * ((float)PI*2f)`
— f4's form), with f64 cos/sin over the f32 angle
(`Utils.cs:1665-1668`); the e2-b5 f64-multiply form drifted sub-ulp.
Both witnesses re-mirror the same order. With that, every overlap
body is draw-equivalent at the tips (the remaining e2 `/16` probes
differ from `>>4` only for negative coordinates — off-world either
way, and the bodies drop at the merge regardless).

| Sites | e2 commit | f4 commit | Keep-one note |
| ----- | --------- | --------- | ------------- |
| 43251/43278/43365 (KingSlime) | `1cbbcb95d` (09:18) | `23e313f60` (05:50) | f4 first; both thread DustPool+SpawnCtx, both re-mirror the K-1 parity witness. Bodies draw-equivalent (10×4 shrink vel*0.5 !flag3 / 10×4 grow vel*2 / 1×4 body alpha255 scale*1.2 vel*0.5). Take f4's; e2's `king_slime_teleport_ring`/`king_slime_body_drip` emitters + the grow-ring machine witness supplement. |
| 40834/40851/40947 (EterniaCrystal) | `31c91d90` (05:17) | `37d143c75` (03:52) | f4 first; both faithful (nine-gate 264 / 120-pulse 264 / twelve-gate 240, the 0f*NextFloat dead draw + XNA color truncation). Take f4's; e2's machine witnesses supplement. |
| 41367/41396 (core storm) | `31c91d90` | `abff45cf8` (10:14) | e2 first (partial: fan + inner smoke); f4 b6 is the FULL :41336-41403 port (top-level nine draws + the ceil(ai[1]/60) inner loop + SelectRandom(31,229)). Take f4's full body; e2's storm/smoke witnesses re-mirror. |
| 42222 (head fan) | `31c91d90` | `abff45cf8` | e2 first; f4 b6 re-ports with the customData drop note. Draw-equivalent (60..120 one / 120..180 two, NextDouble angle, (27,59)/2 ellipse, Normalize inward velocity). Take f4's; e2's witness supplements. |
| 42675/42873 (eye orbit + fan) | `31c91d90` | `abff45cf8` | e2 first; f4 b6 re-ports. Draw-equivalent (2×229 orbit velocity+=vector5/15; (30,30)/2 ellipse fan). Take f4's; e2's witnesses supplement. |

**Attribution fix (e2 b6):** batch 5's `squidhead_rings` (40100/40110/
40120) was mislabeled — aiStyle 101's only user is **AncientDoom 523**
(`SetDefaults` `:15799`), not Squidhead 521 (aiStyle 86, `:15772`). The
emitter is renamed `ancient_doom_rings`; the real Squidhead sites are
37337/37464 (staged in b6).

## 6. e2 batch-6 delivery (this window)

- **KingSlime trio** 43251/43278/43365 — wired inline in
  `update_king_slime` (threads DustPool+SpawnCtx), the crown gore 734
  cited to d55/f4. Overlaps f4 b2 (§5).
- **NebulaFloater fade-entry** 39524 — wired through the FAITHFUL
  aiStyle-97 machine (`6c424595f`): the stand-in's invented `ai[1]--`
  countdown + scatter pick retired for `AI_AttemptToFindTeleportSpot`
  (range 20, telefrag 12, centred, in-air) + the strike-hook latch
  `:81347`; the fade-out's `localAI[3]` one-shot fires the 20×242 burst,
  the silent `active=false` kill carries `moon_despawn_kill` (the
  death-sound pass now guards the flag).
- **Gap census** :21562-39039 — 9 staged sites (24520, 24850, 24866,
  35157, 35162, 37289, 37307, 37337, 37464), all kind-less or
  unwired-machine, pinned in `npc_dust.rs` for the wiring wave.

Witnesses: 6 emitter + 2 KingSlime machine + 2 floater machine, all
WRITTEN NOT RUN (code-only mode).

## 7. e2 SEAL (this round — lane closed)

**Delivered (batches 1-6 + seal):** 40+ sites across the second-batch
window — b1 Twins + staged Skeletron/Brain (`1da18d196`), b2 the
staged wiring + 4 inline machines (`2bfaa71a4`), b3 style39/41 ten
sites (`98f3337e7`), b4 pillar ambient + dissolve ring/plume
(`1ec45ba28`), b5p1 the pillar 6-burst + 241 tail (`0f0d7036e`), b5p2
twenty sites incl. the crystal/MoonLord/floater machines
(`31c91d902`), b5 fidelity + witness fixes (`6e304b960`/`970cd3495`),
b6 KingSlime trio + faithful aiStyle-97 + 9 staged gap sites + the
AncientDoom relabel (`1cbbcb95d`/`6c424595f`/`9912498fc`), seal fix
`d5ae8725c`. Witness census at the seal tip: `npc_dust.rs` 21 emitter
witnesses + `npc.rs` `w4_e2_batch2/5_tests` 10 machine witnesses + the
`s1_d_3_moonlord_rng_witnesses` module's 2 floater witnesses (the
module's other 6 are main-inherited S1-D-3 pins e2 re-derived) — all
WRITTEN NOT RUN, no cargo/tests/rustfmt/graphify (code-only mandate).

**Nothing owed by e2:** the overlap bodies drop at the merge per §5
(f4 keeps); the 9 gap sites + the b5 staged kinds (SolarGoop 519,
AncientLight 522, AncientDoom 523, SandElemental 541) stay STAGED with
their witnesses — the wiring wave owns the machine arms (the
`npc_ai_style9` solar-flare precedent); the MartianTurret plan seam
(`npc_ai_style73.rs::turret_materialize_dust_plan`) composes with the
staged `turret_materialize_ring` emitter. The floater fade-entry is
WIRED faithful (no stand-in remains).

**Cross-lane receipts:** d54 bible §8.2's NPC row (927+5) re-counted
direct-path this round — 836 ND + 16 NDP + 75 NDD = 927, +5 CloneDust;
the window split 159 (<50000) / 87 (50000-69999) / 12 (70000-81653) /
629 (HitEffect 81654-89834, +2 Clone) / 40 (>89834) matches the bible
exactly. The gap census :21562-39039 holds 49 ND-family lines, ALL
claimed by the §2 rows: e2 b1/b2 Twins 6 + Skeletron 2 + Golem-head 2
+ Brain 1 + MoonLeach 2 + Dutchman 1, b3 style39/41 10, b6 staged 9;
c4 style44 3, style56 1, frost-boss-b 1, style74 7, saucer style75 4
- 49/49, zero unclaimed. Family bible: `342-npc-dust-family-bible.md`.