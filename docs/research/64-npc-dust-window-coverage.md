# 64 — NPC.cs dust-site window coverage

> W4-e2 batch 6 · 2026-11-21 · code-only census. Which `Dust.NewDust*`
> windows in `decompiled/Terraria/NPC.cs` are claimed by which lane, at
> the fetched branch tips. Companion to d54f's `131-dust-master-status.md`
> (the Main/world-lane keep-one register); this doc is the **NPC.cs AI +
> HitEffect dust** slice. Site id = the C# `Dust.NewDust*` line number.
> "Wired" = the emitter rolls the live `DustPool` on the `UpdateNPCs`
> stream; "staged" = the body + RNG order are pinned in a pure emitter,
> the machine wiring lands with a later wave; "plan" = a count/geometry
> seam the dust lane fills.

## 1. Lane tips (fetched 2026-11-21)

| Lane | Branch | Tip | Scope |
| ---- | ------ | --- | ----- |
| d52 | task/W4-d52-dust-sites-npc | `231b31555` | first-40 AI lane (Golem/EoC/caster/sphere) |
| e2 | task/W4-e2-dust-sites-npc-2 | `9912498fc` | second batch (this doc's lane) |
| f4 | task/W4-f4-dust-npc-3 | `bcf917437` | third batch + wave E gore |
| g61 | task/W4-g61-npc-spawn-dust | `b5ff1803d` | spawn/teleport-in/transform visuals |
| g62 | task/W4-g62-npc-hit-dust | `32af98b44` | HitEffect every-strike dust |
| g59 | task/W4-g59-npc-death-effects-1 | `29c13d103` | HitEffect death halves (dust-only kinds) |
| g60 | task/W4-g60-npc-death-effects-2 | `19431e385` | death-family gore/dust matrix |
| f5 | task/W4-f5-dust-proj-3 | `d0619d8e9` | wave D classic-mob gore chain (853/853) |
| f10 | task/W4-f10-gore-sites-3 | `05f40324e` | wave C HitEffect gore table |

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
| 50268/50284 | Wyvern fade | `npc_ai_wyvern.rs` (wyvern lane) |
| 50976-51026 | Wyvern 454-459/621-623 fade | `npc_ai_wyvern.rs` |
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

## 5. Overlap register (merger-wave keep-one)

These windows were ported independently by two lanes (the branch list
is the claim record, but the :42873-44037 window was small enough that
f4's third batch and e2's second batch both reached it). The merger
takes ONE body per site; the witnesses supplement.

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