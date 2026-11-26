//! W4-g60 — the NPC death dust/effects lane 2: `NPC.HitEffect`'s per-kind
//! DEATH halves for the vanilla types 231-697 (g59 carries the first
//! half, 1-230), drawn when the death retain picks the corpse up.
//!
//! # The seam (the W4-c12 `style74_death_feed` precedent)
//!
//! The C# runs `HitEffect(hitDirection, num)` inside `StrikeNPC_Inner`
//! (`NPC.cs:81488`) on the striking stage's stream, BEFORE `checkDead`'s
//! DeathSound (`:81559` → `:78186`). The port batches deaths into the
//! NPC pass (`world_sim/npcs.rs`'s death retain), so every draw below
//! rides the `UpdateNPCs` stream — the S1-E-21c recorded deviation (the
//! death-cue precedent). The feed runs BEFORE the death cue, after the
//! moon-event kill reports (neither draws the RNG), under the same
//! carrier guard as the death events (`NPC.cs:78040-78042`'s checkDead
//! guard; the C# HitEffect itself runs per struck segment — the batch
//! model folds the segment draws onto the carrier's death, documented
//! in ANALYSIS). The `life <= 0` halves only: the `life > 0` hit halves
//! stay with the unported strike-side HitEffect (the g62 lane).
//!
//! # Eligibility (the g59 census convention)
//!
//! A kind rides this lane when EVERY top-level arm covering it has a
//! dust-bearing death half, all dust draws precede the gores
//! (`dust_first`), and no side effects (`NewNPC`/`Transform`/`PlaySound`/
//! `AddBuff`/`NewProjectile`/`netMode`/...) sit in the death half. 121
//! ported kinds qualify in 231-697; 388/418 are W4-c12's in-flight death
//! feed (kept out — the merge reconciles), and the 663 Princess's second
//! arm (`NPC.cs:88411`) is DEAD CODE: the else-if chain's town arm
//! (`:88046`) matches 663 first, so only the 25-dust smoke burst fires.
//! The 402 Stardust Worm Head's `oldPos` trail loop (`:83611-83631`)
//! reads the LIVE trail ring (`Npc.old_pos`, `NPC.cs:6119`; TRAILCACHE
//! WIDENING — 36 logical slots for 402, `NPCID.cs:10450`, g62's contract):
//! `Npc::update_trails` ports `UpdateNPC_UpdateTrails`'s mode-7 shift
//! (`NPC.cs:90863-90872`, called at `:90512` after `FindFrame`) — the
//! W4-g60 wiring wave retired the all-zero stand-in. Only a
//! never-updated corpse still reads an all-zero ring (the gates draw
//! until the first `== 0` breaks, exactly the C#'s fresh-NPC shape).
//!
//! # RNG contract (d50's DustPool, S1-E-1/S1-E-4)
//!
//! Every draw hits the caller's stream in C# statement order: the
//! explicit `Main.rand` argument draws FIRST (C# evaluates arguments
//! before the call), then the `NewDust` internals roll inside
//! `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn writes
//! (which read `Main.dust[6000]` when the pool refuses — the sentinel
//! slot exists and takes the writes exactly like the C#). The gore
//! BODIES are NOT drawn here (the d55/e5/f10 gore lane owns `NewGore`):
//! where a gore call carries ARGUMENT draws (`Main.rand.Next(...)` in
//! the call, incl. the conditional-gore gates), those draws ARE consumed
//! here in C# order — the e2 lane's split — so the stream stays
//! faithful; the gore lane must NOT re-draw them (the site comments
//! cite each consumed argument). Constant-argument gore calls draw
//! nothing outside the pool. The `SoundEngine...Stop()` calls
//! (572-575, `:82230-82231`/`:82255`) draw nothing.
//!
//! # Keep-one census vs the d55/e5/f10 gore dispatchers (wiring wave)
//!
//! The gore lanes' `hit_effect`/`hit_effect_b`/`hit_effect_c` re-run
//! death DUST loops for kinds this lane already covers. When their
//! dispatch claims the strike/death slot, the wiring MUST strip those
//! loops for exactly the overlap kinds (keep this feed, keep the gore
//! BODIES — one draw per C# statement):
//! * d55 wave A: 668 (`deerclops_death`'s 120-burst — g60 owns it; the
//!   7 gores stay d55's, constant-argument). 636 is NOT in this lane
//!   (census-ineligible, hit half + gore mix): d55/e5 own it whole —
//!   their `empress_hit_dust_draws` count-plan must become a pool feed
//!   at their wiring, not a second census.
//! * e5 wave B: 551/552-554/555-557/561-563 (`dd2_death_dust`'s 20×5
//!   preamble — g60 owns it; the gore bodies stay e5's). 657/660/667
//!   are NOT in this lane (gore-FIRST arms): e5 owns them whole.
//! * f10 wave C: 558-560/564-565/566-567/568-571/572-575/576-577
//!   (`death_dust_loop`) + 412-417/419/518/416 (the solar pair loops)
//!   — g60 owns the dust AND the consumed argument draws (the 566/567
//!   six `Next(2)` gates, the 576/577 `Next(5)` head pick — f10 re-draws
//!   them today; strip at wiring). 418 stays c12's feed. The e5 551
//!   mirrored twin vs the f10 551 flipped arm ALSO overlap each other
//!   (same `spriteDirection == -1` branch) — keep one there too.
//!
//! # Gore-remainder re-census (the 66 GORE-UNIMPL rows, research/110 §11)
//!
//! The playbook's blanket `GORE-UNIMPL` for the 66 g60-owned remainder
//! kinds was probe-time-stale (f10 tip `a69fb00c`, sites ≤444). The
//! re-read: f10's pushed tip `67c51ec7` now covers `NPC.cs:82206-85334`
//! (batches 24-60 landed the `:83562-85559` continuation — the martian
//! machines 385-387/389/391/399/520, the saucer family 392-395, the
//! pumpkin family 305-314/325/327/344), and f5's wave-D
//! claim (`97c70c97`, tip `b33ba0a9`) owns the WHOLE
//! `NPC.cs:85560-89835` classic-mob chain — gore bodies AND the dust
//! census ("this module ports the death-half dust loops ONLY as the
//! gore bodies' draw census … keep-one at merge"). Of the 66: **24
//! kinds are f10-owned (gore pushed), 32 f5-owned (11 pushed + 21
//! claimed), 10 have NO gore sites at all** (261/265/288/326/330/352/
//! 384/402/410/533 — dust-only arms no gore lane carries because there
//! is nothing to carry; g60's feed is their whole port).
//! **ZERO gore bodies are unclaimed — g60 ports NO gore bodies here**;
//! the ARG draws stay consumed by this feed, exactly once. The strip
//! surface grows: f10's pushed continuation adds 24 overlap kinds
//! (305-314/325/327/344/385-387/389/391/392-395/399/520), f5's wave-D
//! claim adds 32 (the 11 PUSHED re-draws 338-340/342/343/348-351/489/
//! 490 — f5 batches 14/15/24/25/31/32 — + 21 claimed-not-pushed arms
//! past `:86986`). Per-kind table: research/110 §11.
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled
//! by the merge-verify wave, not executed here.

//! W4-g59 — the NPC death DUST lane: `NPC.HitEffect`'s per-kind death-half
//! dust bursts (`life <= 0`), kinds 1-230, batches 1-16 (129 wired C#
//! match types covering 130 census kinds — the 40/41 BoneSerpent
//! segments ride the head's cs_id 39 via d34's Hide-fold; every
//! dust-first clean kind the port's kind set carries, plus the 24 d35
//! row-gated activations of the DOC-SWEEP phase (103's nested-row
//! artifact included), the 23 d34/c4/d23/d37 row-gated activations of
//! batches 11-13, the 17 g8/d120/f42 row-gated activations of
//! batches 14-15 and the 105 base-fold activation of batch 16).
//!
//! `HitEffect` runs inside `StrikeNPC_Inner` (`NPC.cs:81488`), BEFORE
//! `checkDead` (`:81561` → `:78040`); the port batches deaths into the NPC
//! pass (the S1-E-21c recorded deviation), so the feed rides the death
//! retain in `world_sim/npcs.rs` ahead of the death cue — the W4-c12
//! `style74_death_feed` precedent. Every draw hits the `UpdateNPCs` stream.
//!
//! ## Scope — batch 1 (dust-only death arms, zero gore, zero side effects)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 23 | MeteorHead | `:89487-89501` | 50× Next(2)-selected 25/6 + 50× type-6 vel×6 |
//! | 120/137/138 | IlluminantSlime (138) | `:85490-85494` | 50× type-71 alpha 200, vel×1.5 |
//! | 143/144/145 | the snowman trio | `:85370-85375` | 50× type-76 noGravity, scale×1.2 |
//! | 147/184 | IceSlime/SpikedIceSlime | `:85350-85355` | 30× (76 noGravity + 4 tinted) pairs |
//! | 183 | Crimslime | `:85420-85424` | 50× type-136 `this.alpha`/`this.color`, vel×2 |
//! | 210/211 | Bee/BeeSmall | `:85065-85077` | 6× type-150, Next(2) scale gate |
//!
//! ## Scope — batch 2 (the gore-tail kinds; the dust is the whole feed)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 2 | DemonEye | `:86459-86462` | 50× type-5, 2×dir/-2 |
//! | 5 | ServantOfCthulhu | `:87351-87354` | 20× type-5, 2×dir/-2 |
//! | 6/7/13 | EaterOfSouls/Devourer/EoW heads | `:87512`/`:87566`/`:87632` | 50× type-18 alpha |
//! | 10 | GiantWormHead | `:87600-87603` | 10× type-5, 2.5×dir/-2.5 |
//! | 17/18/19/20 | Merchant/Nurse/ArmsDealer/Dryad | `:87662` etc. | 50× type-5, 2.5×dir |
//!
//! Every batch-2 gore tail draws NO arguments (constant types or
//! `this.type` arithmetic — the shimmer/altTexture gates are field reads),
//! so the dust loop is the whole RNG census; the `NewGore` bodies + kicks
//! stay d55's lane (the g63 claim's gore-sibling rule).
//!
//! ## Scope — batch 3 (the skeleton/goblin/Skeletron families)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 21/32 | Skeleton/DarkCaster | `:88537-88540` | 20× type-26, 2.5×dir/-2.5 |
//! | 22 | Guide | `:87932-87935` | 50× type-5, 2.5×dir/-2.5 |
//! | 24 | FireImp | `:89514-89519` | 50× type-6 at velocity, noGravity, vel×2 |
//! | 26/27/28/29 | the goblin quartet | `:89560-89563` | 50× type-5, 2.5×dir/-2.5 |
//! | 35/36 | SkeletronHead/Hand | `:89177-89180` | 150× type-26, 2.5×dir/-2.5 |
//!
//! The batch-3 gore tails also draw NO arguments (the skeleton arm's
//! per-kind ladder, the goblin type ladder `:89564-89596` and the
//! Skeletron RedHat gate `:89181-89208` are constant data / ai-field
//! reads), so the dust loop is again the whole census.
//!
//! ## Scope — batch 4 (the bat/shark/piranha families + the hornet)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 38/54 | Demolitionist/Clothier | `:88335`/`:88252` | 50× type-5, 2.5×dir/-2.5 |
//! | 42 | Hornet | `:89712-89715` | 50× type-18 alpha/color/scale, dir |
//! | 44/45 | UndeadMiner/Tim | `:88537-88540` | 20× type-26, 2.5×dir/-2.5 |
//! | 49/51 | CaveBat/JungleBat | `:85807-85810` | 15× type-5, 2×dir/-2 |
//! | 58 | Piranha | `:86381-86384` | 10× type-5, 2×dir/-2 |
//! | 65 | Shark | `:86740-86743` | 75× type-5, 2×dir/-2 |
//! | 66 | VoodooDemon | `:89803-89806` | 50× type-5, 2×dir/-2 |
//!
//! The batch-4 gore tails draw NO arguments (verified: the hornet's
//! 205/176 ladder, the bat per-kind gores, the piranha/shark/voodoo
//! ladders are all constant data). Shared-arm siblings outside the port's
//! kind set (47/57, 62, 176/205/231-235, 542-545/692) stay cited skips.
//!
//! ## Scope — batch 5 (the wyvern/eye/town families + the leech)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 87 | WyvernHead | `:88921-88932` | 10× type-16 vel×2 + gore ARG draws |
//! | 106/107/108 | BoundWizard/GoblinTinkerer/Wizard | `:87880-87883` | 50× type-5, 2.5×dir/-2.5 |
//! | 111 | GoblinArcher | `:89560-89563` | 50× type-5, 2.5×dir/-2.5 |
//! | 117 | LeechHead | `:87446-87449` | 10× type-5, dir/-1 |
//! | 123/124 | BoundMechanic/Mechanic | `:87912-87915` | 50× type-5, 2.5×dir/-2.5 |
//! | 190/191 | CataractEye/SleepyEye | `:86459-86462` | 50× type-5, 2×dir/-2 |
//!
//! The Wyvern (87) is the FIRST arm with gore ARGUMENT draws: the
//! `Next(1,4)` gore count + the per-gore `Next(11,14)` type roll
//! (`:88927-88930`) are consumed in order here; the `NewGore` bodies +
//! kicks stay d55's (the g63 claim's gore-sibling rule). The other
//! batch-5 tails draw no arguments.
//!
//! ## Scope — batch 6 (the eye trio, the dye/party girls, the pirates)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 192/193/194 | Dialated/Green/PurpleEye | `:86459-86462` | 50× type-5, 2×dir/-2 |
//! | 207/208 | DyeTrader/PartyGirl | `:88378`/`:88401` | 50× type-5, 2.5×dir/-2.5 |
//! | 212-216 | the pirate family | `:88458-88461` | 50× type-5, 2.5×dir/-2.5 |
//!
//! All batch-6 gore tails draw NO arguments (the eye per-kind types and
//! the pirate ladder `:88462-88501` are constant data).
//!
//! ## Scope — batch 7 (the final present-clean kinds)
//!
//! | C# type(s) | port kind(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 222 | QueenBee | `:87491-87494` | 50× type-18 alpha/color/scale, dir |
//! | 225 | UmbrellaSlime | `:85038-85041` | 50× type-4 tinted (0,80,255,100) |
//! | 227/228 | Painter/WitchDoctor | `:88086`/`:88109` | 50× type-5, 2.5×dir/-2.5 |
//!
//! All batch-7 gore tails draw NO arguments (the Queen Bee's 6-gore
//! ladder `:87495-87500`, the Umbrella's 314 and the painter/witch
//! shimmer gates are constant data / field reads).
//!
//! ## Scope — batches 8-10 (the DOC-SWEEP: the d35 row-gated activations)
//!
//! W4-d35's 33-kind SetDefaults wave (`eb0f1005`, kinds 73-140) landed 24
//! kinds inside this lane's 107-kind UNWIRED remainder; 120/137 were
//! ALREADY wired (batch 1's illuminant arm + batch 5's — their arms fire
//! once d35's cs_id rows merge). 112 VileSpit's whole-arm burst
//! (`:85395-85408`) fires on EVERY hit — g62 owns it (`npc_hit_dust.rs`,
//! the `:85399` site). The remaining 24 activate here (103 rides the
//! shared jellyfish arm's own color row — the census's nested-row
//! artifact re-checked against the C#):
//!
//! | batch | C# type(s) | death arm | shape |
//! | --- | --- | --- | --- |
//! | 8 | 73 GoblinScout | `:89560` | 50× type-5 (the goblin arm) |
//! | 8 | 75 Pixie | `:85527-85531` | 50× type-55 alpha 200 vel×2, dust-only |
//! | 8 | 77/110 ArmoredSkeleton/SkeletonArcher | `:88537` | 20× type-26 |
//! | 8 | 78/79/80 the mummy trio | `:88946-88951` | 20× type-31 scale 1.5 vel×2 noGrav |
//! | 8 | 82 Wraith | `:88971-88976` | 20× type-54 alpha 50 scale 1.5 vel×2 noGrav |
//! | 8 | 83/84 CursedHammer/EnchantedSword | `:87152-87163` | 20× type-31 + 6 gore ARG draws |
//! | 9 | 85 Mimic | `:88785` | 20× ai[3]-selected 7/10/37 |
//! | 9 | 86 Unicorn | `:87705` | 50× type-5 |
//! | 9 | 93 GiantBat | `:85807` | 15× type-5 (the bat arm) |
//! | 9 | 94 Corruptor | `:87512` | 50× type-18 alpha/color/scale |
//! | 9 | 95/96/97 the digger trio | `:87616` | 10× type-5 2.5×dir |
//! | 9 | 98/99/100 the seeker trio | `:87583` | 50× type-18 alpha/color/scale |
//! | 10 | 101 Clinger | `:87547` | 50× type-18 alpha/color/scale |
//! | 10 | 103 GreenJellyfish | `:85554-85557` | 25× type-4 tinted (70,225,140,100), dust-only |
//! | 10 | 122 Gastropod | `:85509-85513` | 50× type-72 alpha 200 vel×1.5, dust-only |
//! | 10 | 140 PossessedArmor | `:89085-89090` | 20× type-54 alpha 50 (death-only gate) |
//!
//! The 83/84 pair is the SECOND gore-ARGUMENT arm: each of the 3 gores
//! draws a `Next(-2, 3)` x/y velocity pair (`:87158`/`:87160`/`:87162`) —
//! 6 draws consumed here in C# order, the bodies stay d55's (the g63
//! rule). Every other batch-8-10 tail draws NO arguments (verified
//! against the C#: constant types / `this.type` arithmetic / hitDirection
//! / ai-field reads). The d35 kinds NOT activated: 74/102 (gore-first
//! `:85942`), 104/109 (the zombie arm's NewNPC), 116 (NewNPC), 121
//! (NewNPC), 112 (g62's every-hit).
//!
//! ## Scope — batch 11 (the d34 row-gated activations, 10 kinds)
//!
//! W4-d34's pushed tip `5f3435a8` (phases 1-3, kinds 1-70) landed the
//! cs_id hand rows inside this lane's UNWIRED remainder. The sweep
//! re-intersected the 83-kind row-gated set against EVERY pushed roster
//! tip (d34/d35/d37/d38/d40/d41/d42 + the c4/c8/c10/e26/d23 kind-claims):
//! d34 intersects 14 clean kinds, of which 31/34/37/39/40/41/43/47/48/56
//! activate here (the 40/41 BoneSerpent segments ride d34's Hide-fold
//! onto the head's cs_id 39 — the feed sees 39, the C# arm reads the RAW
//! type, all three draw the same 20×26 burst); 57/61/62/63 follow in
//! batch 12.
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 31 AngryBones | `:88537` | 20× type-26 (the big skeleton arm, 21/32's) |
//! | 34 CursedSkull | `:89132-89141` | 15× (15 noGrav/noLight + 26 noLight) pairs, -vel×0.2, vel×1.3, dust-only |
//! | 37 OldMan | `:88232-88235` | 50× type-5 2.5×dir (the town shape) |
//! | 39/40/41 BoneSerpent | `:89109-89112` | 20× type-26 (40/41 fold onto 39) |
//! | 43/56 ManEater/Snatcher | `:89752-89755` | 50× type-40 alpha/color scale-1.2 |
//! | 47 CorruptBunny | `:86381-86384` | 10× type-5 (the piranha arm, 58's) |
//! | 48 Harpy | `:89786-89789` | 50× type-5 2×dir |
//!
//! All batch-11 gore tails draw NO arguments (the skeleton ladder's
//! generic 42/43/43/44/44 tail, the `this.type - 39 + 67` arithmetic, the
//! 72/72 + 78/79 + 80/81 constants, the Old Man's shimmer field-read
//! gates). The d34 kinds NOT activated: 46 Bunny (gore-first `:85942`).
//!
//! ## Scope — batch 12 (the d34 tail + c4/d23/d37 activations, 10 kinds)
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 57 CorruptGoldfish | `:86381` | 10× type-5 (the piranha arm, d34) |
//! | 61 Vulture | `:86617-86620` | 50× type-5 2×dir (d34) |
//! | 62 Demon | `:89803` | 50× type-5 (the voodoo arm, d34) |
//! | 63 BlueJellyfish | `:85554-85557` | 25× type-4 tinted (50,120,255,100), dust-only (d34; the shared arm's DEFAULT color row — 64 stays row-gated) |
//! | 157 Arapaima | `:86514-86517` | 50× type-5 2×dir (c4 `22cec16f`) |
//! | 175 AngryTrapper | `:89769-89772` | 50× type-40 alpha/color scale-1.2 (d23 `19109424`, its own arm) |
//! | 217/218/219 beetles | `:87462-87465` | 50× type-18 alpha/color/scale (d37 `750d8e44`) |
//! | 226 FlyingSnake | `:85807` | 15× type-5 (the bat arm, d37) |
//!
//! All batch-12 gore tails draw NO arguments (the 84/86-88/93/94/94/
//! 186/187/227/228 constants, the beetle ladder's `this.scale`, the
//! snake's 317-319). The 64 PinkJellyfish tint (225,70,140) lands with
//! its kind row (d36's band, unpushed).
//!
//! ## Scope — batch 13 (the d37 tail, 3 kinds — the rowed set COMPLETE)
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 220/221 SeaSnail/Squid | `:88513-88516` | 20× type-151 2.5×dir scale-0.7 (d37) |
//! | 229 Pirate | `:88129-88143` | the IsShimmerVariant dust SPLIT: 50× type-180 at velocity vel×2 noGrav scale-1.4 vs 50× type-5 2.5×dir (d37) |
//!
//! The Pirate's shimmer gate reads `variation_index == 1 &&
//! shimmer_town_transform(229)` (the `NPC.cs:6713-6723` IsShimmerVariant
//! composition; 229 IS a ShimmerTownTransform member). The altTexture
//! gore gate is a field read + constants — no argument draws.
//!
//! ## Scope — batch 14 (the g8/d120/f42 row-gated activations, part 1)
//!
//! ACTIVATIONS-2 re-intersection: ALL 355 pushed task refs re-probed
//! (33 unique data/bestiary trees, scripted cs_id hand-row + bestiary-
//! fallback extraction per tip). The 45-no-row set intersects THREE
//! tips: g8 `82c93cec` (lane 1's <=230 machine wiring rows — 13 no-row
//! kinds + the 64 PinkJellyfish artifact), d120 `c5e68a80` (the four
//! town kinds 142/160/178/209, the landed truth per the d36 keep-one
//! note), f42 `75f055fc` (the Antlion 69 bestiary row). d34 stays
//! `5f3435a8`; d42's 51-kind wave `9b22c091` is NOW PUSHED (the old
//! probe saw WIP `88df5809`) but its band is 578-630 — intersect ∅;
//! d33 `b08acf46` dragonflies 595-601 / c9 `2042d77f` snails 359/360/
//! 655/677 — intersect ∅ (>230, g60's). d36 `0eb4fee9` is census-only,
//! NO kind rows.
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 69 Antlion | `:86562-86565` | 50× type-250, 2×dir/-2 (f42+g8 rows) |
//! | 142/178 SantaClaus/Steampunker | `:88163`/`:88186` | 50× type-5, 2.5×dir/-2.5 (d120 rows) |
//! | 150/152 IceBat/GiantFlyingFox | `:85807` | 15× type-5 (the bat arm, g8 rows) |
//! | 164/165 WallCreeper pair | `:87783-87786` | 50× type-18, 2.5×dir/-2.5 (g8 rows) |
//!
//! 158 VampireBat rides the 82 wraith arm (`82 | 158`, batch 15 — g8's
//! row); 179 CrimsonAxe rides the 83/84 cursed arm (`83 | 84 | 179`,
//! batch 15 — the SAME 6 gore-ARGUMENT draws). All batch-14 gore tails
//! draw NO arguments (the antlion's 97/98, the town shimmer gates, the
//! bat 164/165 + 175/176/176, the creeper 207/208×4 are constants /
//! field reads).
//!
//! ## Scope — batch 15 (the g8/d120 tail, part 2 — the set COMPLETE)
//!
//! | C# type(s) | death arm | shape |
//! | --- | --- | --- |
//! | 158 VampireBat | `:88971` | 20× type-54 (the wraith arm, g8 row) |
//! | 160 Truffle | `:88272-88275` | 50× type-26 town shape (d120 row) |
//! | 169 IceElemental | `:88830-88836` | 100× type-92 scale=1.5 override, vel×2 noGrav, dust-only (g8 row) |
//! | 174/182 Herpling/FloatyGross | `:86419-86422` | 50× type-5 `this.alpha` (g8 rows) |
//! | 177 Derpling | `:85733-85736` | 15× type-5 (g8 row) |
//! | 179 CrimsonAxe | `:87152-87163` | 20× type-31 + the 6 gore ARG draws (the cursed arm, g8 row) |
//! | 195/196 LostGirl/Nymph | `:86797-86800` | 75× type-5 (g8 rows) |
//! | 209 Cyborg | `:88438-88441` | 50× type-5 town shape (d120 row) |
//!
//! All batch-15 gore tails draw NO arguments beyond the cursed arm's
//! 6 (the truffle/cyborg shimmer ladders, the herpling 225/226 +
//! 238/239, the derpling 234/235/236/236, the nymph 254/255/256 ladder
//! are constants / field reads). The ACTIVATIONS-2 newly-rowed set is
//! COMPLETE: 17 kinds (g8 13 + d120 4, the Antlion 69 double-claimed
//! by f42).
//!
//! ## Scope — batch 16 (the 15-base-fold head-wiring check)
//!
//! The base's `entry_for_npc` folds (bestiary.rs): 8/9→7 Devourer,
//! 11/12→10 GiantWorm, 14/15→13 EoW, 88-92→87 Wyvern, 118/119→117
//! Leech, 105→107 GoblinTinkerer, 135/136/139→134 Destroyer. The
//! heads 7/10/13/87/107/117 are ALL wired (batches 2/5) — the folded
//! segments fire through the head's match, no new activation; the
//! C# arms cover the raw segment ids with the SAME burst (`:87556`
//! 7-9, `:87590` 10-12, `:87622` 13-15, `:88909` 87-92, `:87436`
//! 117-119, `:87870` 105-108 — verified). 105 joins the
//! `105 | 106 | 107 | 108` arm as the raw-id belt-and-braces. The
//! 134 Destroyer head's death arm is NOT dust-first (gore-156 ungated
//! + the Next(2) gate) — UNWIRED here, f5 wave D's
//! `destroyer_death`/`probe_death` own it (keep-one at merge), so
//! 139 stays row-gated. The fold citations land on the head arms.
//!
//! ## The row-gated remainder after batches 14-16 (the census doc)
//!
//! ACTIVATIONS-2 re-intersected the 60-kind remainder (15 base-present
//! folds + 45 no-row) against ALL 355 pushed task refs (33 unique
//! data/bestiary trees, the scripted cs_id hand-row + bestiary-fallback
//! extraction): the 45-no-row set intersects g8 `82c93cec` (13 kinds),
//! d120 `c5e68a80` (142/160/178/209) and f42 `75f055fc` (69, the
//! double-claim with g8) — ALL 17 activated in batches 14-15. The
//! roster tips re-read: d34 stays `5f3435a8` (no advance), d35
//! `e5558b0a` (docs-only past `7d131be3`), d36 `0eb4fee9` (census, NO
//! kind rows), d37 `bdd59afb` (docs past `08f7d25c`), d38 `6f9dd02d`,
//! d40 `8fe1d6d3`, d41 `4ed22f0b` (claim), d42 `9b22c091` — the
//! 51-kind wave NOW PUSHED (the batch-13 probe saw WIP `88df5809`),
//! its band 578-630 intersects ∅; d33 `b08acf46` dragonflies 595-601,
//! c9 `2042d77f` snails 359/360/655/677, f42's desert band 354-362
//! (546/580/581/630 etc.) — ALL >230, g60's per research/110.
//!
//! The 15-base-fold head-wiring check (batch 16): the base's
//! `entry_for_npc` folds 8/9→7, 11/12→10, 14/15→13, 88-92→87,
//! 118/119→117, 105→107 — every head ALREADY wired (batches 2/5), the
//! arms fire through the head's match, no new activation (105 added
//! to the arm as the raw-id belt-and-braces; the C# `:87870` arm
//! covers 105-108 with the same burst). **139 Probe folds 134 — the
//! Destroyer head's death arm (`:89266-89323`) is NOT dust-first (the
//! ungated gore-156 + the Next(2) gate wrap the 10×31 + 5×(6+6)
//! burst) and stays UNWIRED in this lane**: f5 wave D `1b6a1979`
//! ports `destroyer_death`/`probe_death` (`npc_hit_effect_d.rs`,
//! sites 776-786) INCLUDING the dust census — the keep-one at merge
//! (f5's dispatcher is unwired, `#![allow(dead_code)]`, the strike-
//! path claim is their wiring wave). The 139 Probe arm (`:89210-
//! 89244`) is the same shape (10×31 + 5 pairs + 4×Next(61,64) ARG
//! draws). Both stay row-gated here; no double-consume.
//!
//! **43 census kinds stay row-gated UNWIRED** (the S1-F-9b convention):
//! 13 base-present folds riding wired heads (8/9/11/12/14/15/88-92/
//! 118/119 — no new activation needed; 105 activated batch 16) + 139
//! Probe (the unwired Destroyer head, f5's keep-one) + 112 (g62's
//! every-hit re-own) + 28 no-row kinds (68 DungeonGuardian + the d36
//! band 141/148/149/153/154/155/159/163/166/167/168/170/171/173/176/
//! 180/181/185/197/198/199/201-206 — d36's census `0eb4fee9` pushes
//! NO kind rows yet).
//! 64 PinkJellyfish is NOT in the dust-first clean census (its
//! color-only nested `:85537` row is the g59_order artifact) — g8's
//! row landed, it rides 63's arm shape with its own (225,70,140) tint
//! when the merge composes; recorded, not activated here (the census
//! exclusion stands). **129 wired match types / 130 census kinds**
//! across 16 batches (the 40/41 fold accounts for the gap).
//!
//! **Boundary check (g60 research/110): every activated cs_id ≤ 229 ≤
//! 230 — ZERO kinds > 230, the boundary stays CLEAN, nothing flags to
//! the g60 lane.** The newly-rowed >230 siblings (f42's 546/580/581/
//! 630, d120's 579 BartenderUnconscious, d42's 578-630 wave, d33's
//! 595-601, c9's snails) are ALL g60's band — flagged here for the
//! g60-lane per research/110, none activated.
//!
//! ## The lane's remainder (the census's absent kinds)
//!
//! The 173-kind dust-first clean census had 109 kinds with NO port kind
//! row at batch 7's close; the DOC-SWEEP re-census counted 107 UNWIRED
//! (the 66 landed C# types cover 66 of the 173). After batches 8-10, 83
//! stayed row-gated; batches 11-13 activated 23 (the d34/c4/d23/d37
//! rows) leaving 60; batches 14-16 activated 17 + the 105 fold, leaving
//! **43 row-gated** (the S1-F-9b row-gate convention — see the remainder
//! section above); the census rows + arm citations live in
//! docs/ANALYSIS.md. The 20 dust-only kinds' still-absent members
//! (141/204) are the same remainder (34/63 activated in batches 11/12,
//! 169 batch 15, 75/120/122/137 above, 112 re-owned g62).
//!
//! The census (the full 1-230 walk, 173 dust-first clean kinds) and the
//! batch split live in docs/ANALYSIS.md "The NPC death dust table (W4-g59)".
//! The gore-bearing death arms are d55's lane (`npc_hit_effect_a.rs`,
//! ec78d152); the 231-697 band is W4-g60; death SPAWNS are W4-g61; the
//! `life > 0` hit halves are W4-g62. The 112/666 whole-arm burst
//! (`:85395-85408`) fires on EVERY hit, not death-only — g62's.
//!
//! ## RNG contract (the d50 DustPool contract)
//!
//! Argument draws first (the Next(2) selectors), then the pool internals
//! inside `DustPool::new_dust` (S1-E-1: the 6 default-block rolls; type 6
//! adds the tweak-chain `Next(-10, 6)`, `Dust.cs:214-220`; types
//! 4/18/25/71/76/136/150 hit no tweak). A cull refusal or a pressure-band
//! gate consumes exactly the C#'s draws (the pool owns them). The
//! post-spawn writes land on the returned slot, the sentinel included —
//! the C# writes `Main.dust[6000]` too (the `npc_ai_style56.rs` precedent).
//! Batch 1's types are all ChildSafety-safe; batch 2's type 5 IS dangerous
//! (`ChildSafety.cs`'s {5,227,273,237,248,249,250,251} — the
//! `DANGEROUS_DUST` set), but the re-type arm's draws are pool-owned and
//! the blood toggle defaults ON (`Disabled = true`, no re-type).
//!
//! ## Documented stand-ins
//!
//! * `hitDirection` rides [`Npc::last_hit_dir`] (recorded at the three
//!   strike seams; 0 for an argument-less `HitEffect()` — the C# default,
//!   `NPC.cs:81654`). KEEP-ONE (c12/g60/g59): the SAME field is staged
//!   independently on all three branches (`task/W4-c12-style74-martian-
//!   drone`, `task/W4-g60-npc-death-effects-2`, this lane) — one
//!   `pub last_hit_dir: i32` at `npc.rs` with the same three recording
//!   seams (hurt/hurt_from_projectile/strike_pooled) + the c12 bubble-
//!   fuse reset; the merge keeps ONE declaration, every feed reads it.
//!   None of the three re-records it elsewhere.
//! * 183's `this.color`: `default(Color)` — the SetDefaults reset
//!   (`NPC.cs:8526`) is the ONLY write for 183 (the paint-blend ladder
//!   `:60577-60692` is type-1-only, `AI_001_Slimes`). The port's `Npc`
//!   carries no color field; when the slime paint lane lands, the feed
//!   reads it here.
//! * 183's `this.alpha` reads [`Npc::alpha`] — the ctor carries the C#
//!   `alpha = 55` row (`NPC.cs:11342`, added with batch 1).
//! * Batch 2's type-18 trio (6/7/13) reads `this.alpha`/`this.scale` off
//!   the NPC. The EoW head's `alpha = 255` SetDefaults row (`:8921`,
//!   body/tail `:8941`/`:8961`) is the spawn-fade window — NOW PORTED
//!   (the DOC-SWEEP): the ctor carries 255 for 13/14/15 and the worm
//!   pass runs the `AI_006` tail (`NPC.cs:52251-52271`): the 14/15 gate
//!   on the leader's `alpha < 85`, the 2× type-14 fade dust while
//!   `alpha > 0 && life > 0`, the movement-gated `alpha -= 42` floor 0.
//!   The feed reads the LIVE alpha (255 → 0 over the fade, then the
//!   steady-state 0). NOTE: the module doc's old `:50998` citation was
//!   the CultistDragon (454-459) arm — the EoW's own fade is `:52251-
//!   52271` (re-derived in the sweep). The 6/7 pair writes no alpha row
//!   (the Devourer's `scale *= 1.3` `:8784` is the only family tweak),
//!   so their feed keeps reading the ctor 0.
//! * The 143/144/145 trio shares ONE arm; the port's bestiary id rows for
//!   SnowBalla/MysterySnowman (144/145 vs `NPCID.cs:10965-10967`'s
//!   MisterStabby/SnowBalla naming) are dust-neutral — every id in the
//!   trio draws the same burst.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::npc::{Npc, NpcKind};

/// The per-death inputs the C# reads off `this` — snapshotted by the
/// death retain before the pool borrows. `hit_dir` is the recorded
/// strike side (`Npc::last_hit_dir`, the W4-c12 field; 0 = no strike /
/// the argument-less `HitEffect()` fuses, `NPC.cs:81654`'s default).
pub(crate) struct DeathFx<'a> {
    /// The vanilla `NPC.type` (the Sharkron2 fold rides the wrapper's
    /// explicit 373 — `cs_id` folds it onto the bestiary's 372 row).
    pub ty: u16,
    pub pos: Vec2,
    pub w: i32,
    pub h: i32,
    pub vel: Vec2,
    pub hit_dir: i32,
    /// `NPC.oldPos` (`NPC.cs:6119`) — the LIVE trail ring the 402 death
    /// arm's loop reads (`:83611-83631`); `Npc::update_trails` shifts it
    /// per NPC update (the W4-g60 wiring wave — the all-zero stand-in is
    /// retired, only a never-updated corpse still reads all-zero).
    /// TRAILCACHE WIDENING (g62's contract, the g60e half): the max-size
    /// `[Vec2; 60]` storage — the loop bounds on `trail_len` (the C#'s
    /// `this.oldPos.Length` AFTER the `SetDefaults` resize,
    /// `NPC.cs:8453-8468`; 402 → 36, `NPCID.cs:10450`).
    pub old_pos: [Vec2; 60],
    /// The resized `oldPos.Length` captured at strike — the death loop's
    /// bound (`:83611`'s `num179 < this.oldPos.Length`, NO min). Same
    /// widening as `Npc.trail_len` (pure function of kind).
    pub trail_len: usize,
    pub rng: &'a mut UnifiedRandom,
    pub dust: &'a mut DustPool,
    pub ctx: &'a SpawnCtx,
    pub world: &'a World,
}

impl DeathFx<'_> {
    /// One `Dust.NewDust(position, width, height, type, sx, sy)` — the
    /// 6-argument default form (alpha 0, default Color, scale 1).
    fn d(&mut self, ty: i32, sx: f32, sy: f32) -> usize {
        self.dust.new_dust(
            self.rng, self.ctx, self.world, self.pos, self.w, self.h, ty, sx, sy, 0,
            Color::default(), 1.0,
        )
    }

    /// The full-argument form.
    #[allow(clippy::too_many_arguments)]
    fn d_full(
        &mut self, pos: Vec2, w: i32, h: i32, ty: i32, sx: f32, sy: f32,
        alpha: i32, color: Color, scale: f32,
    ) -> usize {
        self.dust.new_dust(
            self.rng, self.ctx, self.world, pos, w, h, ty, sx, sy, alpha, color, scale,
        )
    }

    /// The flat N-dust burst at `base.position`, speeds `(sx, sy)`.
    fn burst(&mut self, n: i32, ty: i32, sx: f32, sy: f32) {
        for _ in 0..n {
            self.d(ty, sx, sy);
        }
    }

    /// The tower-mob pair loop: `n` iterations of one `base_ty` dust at
    /// `(hitDirection, -1)` then a gated `second_ty` dust. `gate_mod`
    /// 0 = unconditional; else `Next(gate_mod) == 0`. `full` writes
    /// noGravity/scale 1.5/fadeIn 1/velocity ×3 (the 241/6 + 249/242 +
    /// 241/229 shape); the stardust 17/229 loop has its own lighter
    /// shape ([`Self::stardust_loop`]).
    fn pair_loop(&mut self, n: i32, gate_mod: i32, base_ty: i32, second_ty: i32) {
        let hd = self.hit_dir as f32;
        for _ in 0..n {
            self.d(base_ty, hd, -1.0);
            if gate_mod == 0 || self.rng.next_max(gate_mod) == 0 {
                let i = self.d(second_ty, 0.0, 0.0);
                let d = self.dust.slot_mut(i);
                d.no_gravity = true;
                d.scale = 1.5;
                d.fade_in = 1.0;
                d.velocity *= 3.0;
            }
        }
    }

    /// The stardust 17/229 pair loop (`NPC.cs:83496-83502` shape): the
    /// GATE applies to the 17 (base) dust — `gate_mod` 0 = always,
    /// else `Next(gate_mod) == 0` XOR `gate_ne` (407's `!= 0` form) —
    /// and the 229 ALWAYS spawns with noGravity + velocity ×3 ONLY.
    fn stardust_loop(&mut self, n: i32, gate_mod: i32, gate_ne: bool) {
        let hd = self.hit_dir as f32;
        for _ in 0..n {
            let spawn17 = if gate_mod == 0 {
                true
            } else {
                (self.rng.next_max(gate_mod) == 0) != gate_ne
            };
            if spawn17 {
                self.d(17, hd, -1.0);
            }
            let i = self.d(229, 0.0, 0.0);
            let d = self.dust.slot_mut(i);
            d.no_gravity = true;
            d.velocity *= 3.0;
        }
    }

    /// The 54-smoke loop (250/315, `NPC.cs:88971-88976`): `n` × type-54
    /// at (0,0), alpha 50, scale 1.5, then velocity ×2 + noGravity.
    fn smoke54(&mut self, n: i32) {
        for _ in 0..n {
            let i = self.d_full(self.pos, self.w, self.h, 54, 0.0, 0.0, 50, Color::default(), 1.5);
            let d = self.dust.slot_mut(i);
            d.velocity *= 2.0;
            d.no_gravity = true;
        }
    }

    /// The 226 scale-0.5 loop (the martian machines): `n` × type-226 at
    /// `(-k × hitDirection, -k)`, post-spawn scale = 0.5.
    fn burst226(&mut self, n: i32, k: f32) {
        let hd = self.hit_dir as f32;
        for _ in 0..n {
            let i = self.d(226, -k * hd, -k);
            self.dust.slot_mut(i).scale = 0.5;
        }
    }
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): the unit draw
/// (`NextVector2Unit`, `:1261-1264` — ONE NextFloat, float angle, double
/// cos/sin) THEN a second `NextFloat` radius scale. The
/// `npc_ai_empress.rs` helper's shape, kept local (that one is private).
fn circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
    let r = rng.next_float();
    Vec2::new(unit.x * hw * r, unit.y * hh * r)
}

/// The death retain's entry: snapshot the corpse, run the kind's death
/// half on the `UpdateNPCs` stream. Kinds outside the lane no-op (their
/// HitEffect stays the documented visual skip / the sibling lanes').
pub(crate) fn death_dust_feed(app: &mut crate::app::App, npc: &Npc) {
    // The vanilla type: the Sharkron2 fold is explicit (the style71
    // adapter's precedent — `cs_id`'s bestiary fallback folds 373 onto
    // the Sharkron 372 row, and the shared arm covers both).
    let ty = match npc.kind {
        NpcKind::Sharkron2 => 373,
        _ => npc.kind.cs_id(),
    };
    if !lane_covers(ty) {
        return;
    }
    // `NPC.cs:87222-87225` — the Spore's gate is `life >= 0` (NOT the
    // usual `life > 0`): at exactly life == 0 the arm returns with NO
    // dust. The death retain picks hp <= 0, so the hp == 0 edge skips.
    if ty == 265 && npc.hp == 0 {
        return;
    }
    let (w, h) = npc.size();
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdateNpcs);
    let mut fx = DeathFx {
        ty,
        pos: Vec2::new(npc.x, npc.y),
        w: w as i32,
        h: h as i32,
        vel: Vec2::new(npc.vx, npc.vy),
        hit_dir: npc.last_hit_dir,
        old_pos: npc.old_pos,
        trail_len: npc.trail_len,
        rng,
        dust: &mut app.dust,
        ctx: &ctx,
        world: &app.world,
    };
    death_dust_draws(&mut fx);
}

/// The lane's kind gate (the census's eligible set, minus c12's
/// 388/418). Kept as a table so the wrapper can skip the snapshot for
/// the ~570 uncovered types.
fn lane_covers(ty: u16) -> bool {
    matches!(
        ty,
        668 | 552..=554 | 561..=563 | 555..=557 | 551 | 558..=560 | 566 | 567
            | 576 | 577 | 564 | 565 | 572..=575 | 570 | 571 | 568 | 569 | 550 | 441
            | 588 | 589 | 633 | 368 | 353 | 354 | 369 | 376 | 663 | 342
            | 338..=340 | 343 | 348 | 349 | 350 | 351 | 489 | 490 | 305..=314
            | 352 | 327 | 325 | 326 | 330 | 329 | 315 | 250 | 261 | 288
            | 344 | 262..=264 | 265 | 266 | 267 | 370 | 372 | 373
            | 439 | 381 | 382 | 390 | 383 | 384 | 385 | 386 | 387 | 389
            | 391 | 392..=395 | 399 | 520 | 341 | 471 | 533
            | 518 | 416 | 419 | 412..=414 | 415 | 417 | 427 | 425
            | 428 | 424 | 423 | 421 | 420 | 409 | 410 | 407 | 411 | 402
    )
}

/// The per-kind death halves, in C# source order. Every arm cites its
/// `NPC.cs` window; the dust draws roll the pool, the gore ARGUMENT
/// draws are consumed in place (the module-header split).
pub(crate) fn death_dust_draws(fx: &mut DeathFx) {
    let hd = fx.hit_dir as f32;
    match fx.ty {
        // `NPC.cs:81892-81909` — Deerclops: 120 type-5 dusts from
        // `position - (15,15)`, box `(width+30, height+30)`, scale 1.5,
        // then velocity = `NextVector2Circular(4,4)` (TWO draws AFTER
        // the spawn internals). The seven gores (`:81902-81908`) are
        // constant-argument — the gore lane's, zero draws here.
        668 => {
            let pos = fx.pos - Vec2::new(15.0, 15.0);
            let (w, h) = (fx.w + 30, fx.h + 30);
            for _ in 0..120 {
                let i = fx.d_full(pos, w, h, 5, 0.0, 0.0, 0, Color::default(), 1.5);
                let v = circular(fx.rng, 4.0, 4.0);
                fx.dust.slot_mut(i).velocity = v;
            }
        }
        // `NPC.cs:82088-82108` — the DD2 Goblin trio: 20 type-5 at
        // `(hitDirection, -1)` (the death half keeps the 1× side, NOT
        // the hit half's). The five gores (`:82103-82107`, the
        // 983/1067/1071 tier rows) are constant-argument — zero draws.
        552..=554 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82119-82141` — the DD2 Javelin trio: the same 20×5;
        // the seven gores (`:82134-82140`, 987/1055/1061) draw nothing.
        561..=563 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82152-82170` — the DD2 Bomber trio: the tier pick
        // (`:82154-82162`) precedes the loop in C# but draws nothing;
        // 20×5 then three constant gores.
        555..=557 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82185-82217` — DD2 Betsy: 20×5 at `(hitDirection,-1)`;
        // the ten-gore fan is spriteDirection-branched (`:82191`/
        // `:82204`) but BOTH branches are constant-argument — zero
        // draws outside the dust loop.
        551 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82323-82345` — the DD2 Wyvern trio: 20×5; the seven
        // gores (974/1040/1045 tier rows) are constant-argument.
        558..=560 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82356-82387` — the DD2 Skeleton pair: 20×236, then
        // gore 970 (constant) and SIX `Next(2) == 0`-gated gores
        // (`:82363-82386`: 972×2, 973×2, 971×2). The gate draws are
        // ARGUMENT-side conditional spawns — consumed here in order
        // (the gore bodies stay the gore lane's; it must NOT re-draw
        // the six gates).
        566 | 567 => {
            fx.burst(20, 236, hd, -1.0);
            for _ in 0..6 {
                // `:82363`/`:82367`/`:82371`/`:82375`/`:82379`/`:82383`
                // — each gate draws unconditionally; == 0 spawns the
                // gore (the gore lane reads the same stream position).
                fx.rng.next_max(2);
            }
        }
        // `NPC.cs:82398-82424` — the DD2 Ogre pair: 20×273 (a
        // DangerousDust id — the ChildSafety re-type rolls INSIDE
        // new_dust when armed), then the spriteDirection-branched fan:
        // BOTH branches' first gore carries `1005 + ((Next(5) == 0) ?
        // 1 : 0)` (`:82406`/`:82417`) — ONE Next(5) argument draw
        // consumed here; the remaining five gores are constant. The
        // left branch's two `rotation = PI` writes are gore-side.
        576 | 577 => {
            fx.burst(20, 273, hd, -1.0);
            fx.rng.next_max(5);
        }
        // `NPC.cs:82435-82453` — the DD2 Dark Mage pair: 20×5; the
        // eleven gores (1009..1019) are constant-argument.
        564 | 565 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82228-82242` — the DD2 Kobold Walker pair: the two
        // `SoundEngine...Stop()` calls (`:82230-82231`) draw nothing;
        // 20×5 then five constant gores at Center-(10,10).
        572 | 573 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82253-82267` — the DD2 Kobold Flyer pair: one Stop
        // (`:82255`, no draws); 20×5 then five constant gores.
        574 | 575 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82278-82290` — the DD2 Drakin pair: 20×5; the five
        // gores (1075/979 tier rows) are constant-argument.
        570 | 571 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:82301-82312` — the DD2 Wither Beast pair: 20×5; the
        // five gores (1020-1023) are constant-argument.
        568 | 569 => fx.burst(20, 5, hd, -1.0),
        // `NPC.cs:87952-87960` — the DD2 Bartender: 50×5 at
        // `(2.5×hitDirection, -2.5)`; the five gores are
        // IsShimmerVariant-conditional but BOTH arms constant.
        550 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:87685-87693` — the Tax Collector: the same 50×5; the
        // five shimmer-conditional gores are constant both ways.
        441 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:87972-87983` — the Golfer pair (588/589): 50×5; the
        // altTexture==0 head gore + the four body gores are
        // shimmer-conditional but constant both ways — zero draws.
        588 | 589 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:87995-88021` — the Zoologist: 50×5; the altTexture/
        // shimmer gore picks (`:87999-88015`) are data, not draws.
        633 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:88033-88044` — the Travelling Merchant: 50×5; the
        // altTexture/shimmer gores are constant both ways.
        368 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:88208-88220` — the Stylist pair (353/354): 50×5; the
        // altTexture/shimmer gores are constant both ways.
        353 | 354 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:88048-88074` — the town smoke arm (369 Angler / 376
        // Sleeping Angler / 663 Princess; the IsTownPet/695/696 rows
        // have no port kinds yet): `life <= 0` gates the WHOLE arm;
        // 25 type-31 dusts at (0,0), alpha 100, scale 2, then velocity
        // ×1.4 + noLight + noGravity. The four tombstone gores
        // (`:88058-88073`) each carry `Main.rand.Next(61, 64)` — FOUR
        // argument draws consumed here in order (the gore bodies +
        // their scale/velocity±1 writes stay the gore lane's). The
        // 663's second arm (`:88411`) is dead code — this chain arm
        // matches 663 first (the else-if at `:88046`).
        369 | 376 | 663 => {
            for _ in 0..25 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 31, 0.0, 0.0, 100, Color::default(), 2.0);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 1.4;
                d.no_light = true;
                d.no_gravity = true;
            }
            for _ in 0..4 {
                // `:88058`/`:88062`/`:88066`/`:88070` — the tombstone
                // type pick, argument-side.
                fx.rng.next_range(61, 64);
            }
        }
        // `NPC.cs:86858-86866` — the Gingerbread Man: 50×type-0 at
        // `(2.5×hitDirection, -2.5)`; five constant gores.
        342 => fx.burst(50, 0, 2.5 * hd, -2.5),
        // `NPC.cs:86878-86896` — the Zombie Elf trio: 50×5; the
        // per-type head gore (494/495/496) + three body gores are
        // constant-argument.
        338..=340 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:86908-86915` — the Yeti: 50×5; four constant gores.
        343 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:86322-86330` — the Nutcracker pair (348/349): 10×5
        // at `(2×hitDirection, -2)`; five constant gores (528-532).
        348 | 349 => fx.burst(10, 5, 2.0 * hd, -2.0),
        // `NPC.cs:86362-86369` — the Elf Archer: 10×5; four constant
        // gores (533-536).
        350 => fx.burst(10, 5, 2.0 * hd, -2.0),
        // `NPC.cs:86342-86350` — the Krampus: 10×5; five constant
        // gores (538-542).
        351 => fx.burst(10, 5, 2.0 * hd, -2.0),
        // `NPC.cs:86669-86688` — the Blood Zombie (489; the 586
        // Crimslime shares the arm but has no port kind): 50×5; the
        // type-gated five gores (722-724) are constant-argument.
        489 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:86720-86728` — the Drippler: 50×5, then gores
        // 725/726/727 (constant) and TWO `725 + Main.rand.Next(3)`
        // wing picks (`:86727-86728`) — TWO argument draws consumed
        // here in order.
        490 => {
            fx.burst(50, 5, 2.5 * hd, -2.5);
            fx.rng.next_max(3);
            fx.rng.next_max(3);
        }
        // `NPC.cs:84804-84815` — the Scarecrow decade (305-314): 60×
        // type-194 at `(2×hitDirection, -2)`, scale 1.1; the five
        // gores (441-445) are constant-argument.
        305..=314 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 194, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:84786-84793` — Flocko: 60×67 at `(2×hitDirection,
        // -2)`, post-spawn noGravity; no gores.
        352 => {
            for _ in 0..60 {
                let i = fx.d(67, 2.0 * hd, -2.0);
                fx.dust.slot_mut(i).no_gravity = true;
            }
        }
        // `NPC.cs:84826-84836` — Pumpking: 60×189 scale 1.1; the nine
        // gores (476-484, velocity ×0) are constant-argument.
        327 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 189, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:84847-84857` — Mourning Wood: 60×191 scale 1.1; the
        // seven gores (469-475, velocity ×0) are constant-argument.
        325 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 191, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:84901-84907` — Splinterling: 60×191 scale 1.1; no
        // gores.
        326 => {
            for _ in 0..60 {
                fx.d_full(fx.pos, fx.w, fx.h, 191, 2.0 * hd, -2.0, 0, Color::default(), 1.1);
            }
        }
        // `NPC.cs:88994-89001` — Poltergeist (330; the 316 Pirate
        // ghost shares the arm, no port kind): 50×192 at `(2×hd, -2)`,
        // alpha 100, Color(100,100,100,100), post-spawn noGravity.
        330 => {
            for _ in 0..50 {
                let i = fx.d_full(
                    fx.pos, fx.w, fx.h, 192, 2.0 * hd, -2.0, 100,
                    Color::new(100, 100, 100, 100), 1.0,
                );
                fx.dust.slot_mut(i).no_gravity = true;
            }
        }
        // `NPC.cs:87745-87751` — Hellhound: 50×5; three constant gores.
        329 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:89038-89051` — the Headless Horseman: the 20×54
        // smoke loop, then three gore-99 calls + gore 446 — ALL
        // constant-argument (`(hitDirection, 0)` velocities).
        315 => fx.smoke54(20),
        // `NPC.cs:88971-88982` — Angry Nimbus (250; the 82/158/159/253
        // rows share the arm, g59's half / no port kinds): the 20×54
        // smoke loop; three constant gore-99 calls.
        250 => fx.smoke54(20),
        // `NPC.cs:89065-89079` — Fungi Spore (261; the 254-260/634/635
        // spore rows share the arm, no port kinds): 20×165 at (0,0),
        // alpha 50, scale 1.5, velocity ×2 + noGravity. The three
        // gores are `type != 261`-gated (`:89071`) — 261 spawns NONE.
        261 => {
            for _ in 0..20 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 165, 0.0, 0.0, 50, Color::default(), 1.5);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:85000-85009` — the Dungeon Spirit's death burst (the
        // `npc_ai_style56.rs` cited skip, now drawn): `life <= 0`
        // condition-gated arm; 50×180 at `(velocity.X, velocity.Y)`,
        // then velocity ×2 + noGravity + scale = 1.4 (OVERWRITE — the
        // rolled scale is consumed but replaced).
        288 => {
            for _ in 0..50 {
                let i = fx.d(180, fx.vel.x, fx.vel.y);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.scale = 1.4;
            }
        }
        // `NPC.cs:84876-84890` — Everscream: 60×196 at `(2×hd, -2)`;
        // the two gore loops (508-512, 509-512, velocity ×0) are
        // constant-argument.
        344 => fx.burst(60, 196, 2.0 * hd, -2.0),
        // `NPC.cs:87186-87218` — the Plantera family (262/263/264):
        // 150 iterations; the `type == 265 && Next(3) != 0` gate is
        // FALSE for every family type (265 short-circuits — NO gate
        // draw), so all 150 are type-167 at `(2×hd, -2)`. The 262's
        // nine gores carry `Next(width)`/`Next(height)` ARGUMENT pairs
        // (`:87199-87207`) — 18 draws consumed here; the 263/264 pairs
        // (`:87211-87217`) — 4 each. The gore bodies stay the lane's.
        262..=264 => {
            fx.burst(150, 167, 2.0 * hd, -2.0);
            let n_gores = match fx.ty {
                262 => 9,
                _ => 2,
            };
            for _ in 0..n_gores {
                fx.rng.next_max(fx.w);
                fx.rng.next_max(fx.h);
            }
        }
        // `NPC.cs:87226-87236` — the Spore: `life >= 0` returns (the
        // death half is life < 0); 15 iterations of `Next(3) != 0` →
        // type-166 else type-167, at `(hitDirection, -1)`. The gate
        // draws EVERY iteration (the `npc_ai_style50.rs` count plan's
        // draws, now emitted).
        265 => {
            for _ in 0..15 {
                if fx.rng.next_max(3) != 0 {
                    fx.d(166, hd, -1.0);
                } else {
                    fx.d(167, hd, -1.0);
                }
            }
        }
        // `NPC.cs:87248-87257` — the Brain of Cthulhu: 150×5 at
        // `(2×hd, -2)`; the six gores (396-401) each carry TWO
        // `Next(-30,31)*0.2` velocity ARGUMENT draws (`:87252-87257`)
        // — 12 draws consumed here in order.
        266 => {
            fx.burst(150, 5, 2.0 * hd, -2.0);
            for _ in 0..6 {
                fx.rng.next_range(-30, 31);
                fx.rng.next_range(-30, 31);
            }
        }
        // `NPC.cs:87269-87273` — the Creeper: 20×5 at `(2×hd, -2)`;
        // one constant gore (402).
        267 => fx.burst(20, 5, 2.0 * hd, -2.0),
        // `NPC.cs:84267-84279` — Duke Fishron: 150×5 at `(2×hd, -2)`;
        // the six gores (576/574/575/573/574/575) are constant-
        // argument (the direction offsets are data).
        370 => fx.burst(150, 5, 2.0 * hd, -2.0),
        // `NPC.cs:84216-84227` — the Sharkron pair (372/373): 75×5
        // from `Center - (25,25)`, box 50×50, at `(2×hd, -2)`, then
        // velocity /= 2 (the `npc_ai_style71.rs` death plan's draws).
        // The four gores (583/577/578/579) are constant-argument.
        372 | 373 => {
            let (cx, cy) = (fx.pos.x + fx.w as f32 * 0.5, fx.pos.y + fx.h as f32 * 0.5);
            let pos = Vec2::new(cx - 25.0, cy - 25.0);
            for _ in 0..75 {
                let i = fx.d_full(pos, 50, 50, 5, 2.0 * hd, -2.0, 0, Color::default(), 1.0);
                fx.dust.slot_mut(i).velocity /= 2.0;
            }
        }
        // `NPC.cs:84138-84159` — the Ancient Cultist: 30×5 at
        // `(2.5×hd, -2.5)`, then 10×16 at (0,0) scale 1.5 with
        // velocity ×2 + noGravity. The hood-gore loop: ONE `Next(1,4)`
        // count draw (`:84150`), then per gore ONE `Next(11,14)` type
        // pick (`:84153`) — the count gates how many picks are drawn;
        // the velocity `(hitDirection, 0)` + the ×0.8 write are
        // argument-free/gore-side. The four body gores (902/903/901×2)
        // are constant.
        439 => {
            fx.burst(30, 5, 2.5 * hd, -2.5);
            for _ in 0..10 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 16, 0.0, 0.0, 0, Color::default(), 1.5);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            let n = fx.rng.next_range(1, 4);
            for _ in 0..n {
                fx.rng.next_range(11, 14);
            }
        }
        // `NPC.cs:84290-84300` — Brain Scrambler: 30×227 at
        // `(2.5×hd, -2.5)`; four constant gores (587/590/588/589).
        381 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84311-84320` — Ray Gunner / Scutlix Rider (382/390):
        // 30×227; three constant gores (591-593).
        382 | 390 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84331-84340` — Martian Officer: 30×227; three
        // constant gores (594-596).
        383 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84353-84360` — Force Bubble: 70×226 at
        // `(-2.5×hd, -2.5)`, scale = 0.5 OVERWRITE; no gores.
        384 => fx.burst226(70, 2.5),
        // `NPC.cs:84371-84380` — Gray Grunt: 30×227; three constant
        // gores (597-599).
        385 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84391-84400` — Martian Engineer: 30×227; three
        // constant gores (600-602).
        386 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84413-84423` — Martian Turret (Tesla): 30×226 at
        // `(-1.5×hd, -1.5)`, scale = 0.5; three constant gores
        // (603×2/604).
        387 => fx.burst226(30, 1.5),
        // `NPC.cs:84459-84468` — Giga Zapper: 30×227; three constant
        // gores (904-906).
        389 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84543-84552` — Scutlix: 30×227; three constant gores
        // (607/608 at position + width/2, 609).
        391 => fx.burst(30, 227, 2.5 * hd, -2.5),
        // `NPC.cs:84565-84590` — the Martian Saucer family (392/393/
        // 394/395): 30×228 at `(-1.5×hd, -1.5)`, scale = 0.5 OVERWRITE
        // (228's NewDust tweak — vy re-roll, vx ×0.3, scale ×0.7 —
        // rolls INSIDE new_dust BEFORE the overwrite). The 395's six
        // gores / 393's 616 / 394's 617 are constant-argument; the
        // base 392 spawns no gores.
        392..=395 => {
            let hd2 = fx.hit_dir as f32;
            for _ in 0..30 {
                let i = fx.d(228, -1.5 * hd2, -1.5);
                fx.dust.slot_mut(i).scale = 0.5;
            }
        }
        // `NPC.cs:84513-84532` — the Martian Probe: 10×226 at
        // `(-2.5×hd, -2.5)`, scale = 0.5; the two gores (898/899) ride
        // the ai[0]==3 implosion velocity (argument-free — the gore
        // lane reads `Npc.ai[0]`/`rotation`/`sprite_dir` itself).
        399 => fx.burst226(10, 2.5),
        // `NPC.cs:84436-84448` — the Martian Walker: 30×226 at
        // `(-1.5×hd, -1.5)`, scale = 0.5; five constant gores
        // (851/852/848/849/850).
        520 => fx.burst226(30, 1.5),
        // `NPC.cs:88755-88764` — the Present Mimic: 20×196 (the
        // `num706 = 196` constant, `:88746`) at (0,0); the three gores
        // (61/62/63) carry `(hitDirection, 0)` velocities — data, no
        // draws; the ×0.3 writes are gore-side.
        341 => fx.burst(20, 196, 0.0, 0.0),
        // `NPC.cs:89617-89625` — the Goblin Summoner: 50×5 at
        // `(2.5×hd, -2.5)`; five constant gores (675-678).
        471 => fx.burst(50, 5, 2.5 * hd, -2.5),
        // `NPC.cs:83878-83887` — the Desert Djinn: 40×27 at (0,0),
        // alpha 50, scale 1.5, then velocity ×2 + noGravity + fadeIn 1.
        533 => {
            for _ in 0..40 {
                let i = fx.d_full(fx.pos, fx.w, fx.h, 27, 0.0, 0.0, 50, Color::default(), 1.5);
                let d = fx.dust.slot_mut(i);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 1.0;
            }
        }
        // `NPC.cs:82802-82821` — Solar Spearman / Drakomire Rider
        // (518/416): 20 × (241 + `Next(3)==0`-gated 6 full-write);
        // five constant gores (844/845/847/835/836).
        518 | 416 => fx.pair_loop(20, 3, 241, 6),
        // `NPC.cs:82840-82860` — Solar Selenian: the same 20×(241+6)
        // gated pair; six constant gores (844/845/846×2/847×2).
        419 => fx.pair_loop(20, 3, 241, 6),
        // `NPC.cs:82871-82882` — the Solar Crawltipede family (412/
        // 413/414): 6×241 at `(2.5×hd, -2.5)` then 2×6 at the same
        // speeds, alpha 100, Color.Transparent, scale 2 (NO post
        // writes — the C# passes the full arguments). One gore
        // (`type - 412 + 828`, Scale-default) — constant-argument.
        412..=414 => {
            let hd25 = 2.5 * fx.hit_dir as f32;
            fx.burst(6, 241, hd25, -2.5);
            for _ in 0..2 {
                fx.d_full(fx.pos, fx.w, fx.h, 6, hd25, -2.5, 100, Color::TRANSPARENT, 2.0);
            }
        }
        // `NPC.cs:82901-82916` — Solar Drakomire: 40 × (241 + 6
        // UNCONDITIONAL full-write pair); four constant gores at
        // Center (831/832 ×0.8, 833/834 ×0.9).
        415 => fx.pair_loop(40, 0, 241, 6),
        // `NPC.cs:82935-82957` — Solar Sroller: 30 × (241 +
        // `Next(2)==0`-gated 6); eight constant gores (837/838×3/
        // 839×2/840×2).
        417 => fx.pair_loop(30, 2, 241, 6),
        // `NPC.cs:83103-83123` — Vortex Alien: 10×241 at `(hd, -1)`,
        // then 20 × (`SelectRandom(229,229,240)` — ONE `Next(3)` pick
        // BEFORE the spawn, `Utils.cs:2459-2462` — then noGravity,
        // scale = 1.25 + NextFloat, fadeIn 0.25, velocity ×3, noLight).
        // Four constant gores at Center (802-805).
        427 => {
            let hd1 = fx.hit_dir as f32;
            fx.burst(10, 241, hd1, -1.0);
            for _ in 0..20 {
                let pick = fx.rng.next_max(3);
                let ty = [229, 229, 240][pick as usize];
                let i = fx.d(ty, 0.0, 0.0);
                let extra = fx.rng.next_float();
                let d = fx.dust.slot_mut(i);
                d.no_gravity = true;
                d.scale = 1.25 + extra;
                d.fade_in = 0.25;
                d.velocity *= 3.0;
                d.no_light = true;
            }
        }
        // `NPC.cs:83224-83245` — Vortex Rifleman: 20 × (241 +
        // `Next(3)==0`-gated 229 full-write); seven constant gores
        // (796/797/798/809×2/810×2).
        425 => fx.pair_loop(20, 3, 241, 229),
        // `NPC.cs:83256-83272` — Vortex Headcrab: 10 × (241 + a
        // `Next(2)`-gated EITHER 229 OR 240 — the gate draws, then the
        // branch picks the type, NO post writes); two constant gores
        // (806/807, velocity ×0.8).
        428 => {
            let hd1 = fx.hit_dir as f32;
            for _ in 0..10 {
                fx.d(241, hd1, -1.0);
                if fx.rng.next_max(2) == 0 {
                    fx.d(229, 0.0, 0.0);
                } else {
                    fx.d(240, 0.0, 0.0);
                }
            }
        }
        // `NPC.cs:83291-83310` — Predictor: 20 × (249 +
        // `Next(3)==0`-gated 242 full-write); five constant gores
        // (793/794×2/795×2).
        424 => fx.pair_loop(20, 3, 249, 242),
        // `NPC.cs:83329-83343` — Nebula Beast: 20 × (249 + 242
        // UNCONDITIONAL full-write); three constant Center gores
        // (790/791 ×0.8, 792 ×0.9).
        423 => fx.pair_loop(20, 0, 249, 242),
        // `NPC.cs:83362-83376` — Brain Suckler: 10 × (249 + 242
        // unconditional); three constant Center gores (785-787).
        421 => fx.pair_loop(10, 0, 249, 242),
        // `NPC.cs:83395-83409` — Nebula Floater: 10 × (249 + 242
        // unconditional); three constant Center gores (782-784).
        420 => fx.pair_loop(10, 0, 249, 242),
        // `NPC.cs:83494-83506` — Stardust Spider (big): 20 × (17 +
        // 229 noGravity/×3 — the LIGHT stardust shape); three constant
        // Center gores (775/776 ×0.8, 777 ×0.9).
        409 => fx.stardust_loop(20, 0, false),
        // `NPC.cs:83521-83530` — Stardust Spider (small): 5 × (17 +
        // 229); no gores.
        410 => fx.stardust_loop(5, 0, false),
        // `NPC.cs:83545-83563` — Stardust Jellyfish: 20 × (`Next(3)
        // != 0`-gated 17 + ALWAYS 229); six constant gores (778/779
        // Top ×0.8, 780/781/780/781 Center ×0.9).
        407 => fx.stardust_loop(20, 3, true),
        // `NPC.cs:83647-83662` — Stardust Soldier: 20 × (17 + a
        // `Next(2)==0`-gated 229 with noGravity ONLY — no ×3); five
        // constant gores (772/773×2/774×2).
        411 => {
            let hd1 = fx.hit_dir as f32;
            for _ in 0..20 {
                fx.d(17, hd1, -1.0);
                if fx.rng.next_max(2) == 0 {
                    let i = fx.d(229, 0.0, 0.0);
                    fx.dust.slot_mut(i).no_gravity = true;
                }
            }
        }
        // `NPC.cs:83599-83632` — the Stardust Worm Head: 5 ×
        // (`Next(3)==0`-gated 17 + ALWAYS 229 noGravity/×3), then the
        // oldPos trail loop over the LIVE ring (`Npc.old_pos`,
        // `Npc::update_trails`'s mode-7 shift — the W4-g60 wiring wave):
        // TRAILCACHE WIDENING — the bound is `num179 < this.oldPos.Length`
        // (`:83611`, NO min), the RESIZED length: 402 walks up to 36
        // slots (`trail_len`, `NPCID.cs:10450`), NOT the 60-slot storage.
        // Each slot draws `Next(4)` FIRST (`:83613`), `!= 0` continues;
        // on `== 0` a zero slot breaks (`:83617-83620`), else TWO gated
        // pairs spawn at `oldPos[i]` (`:83621-83630`, the same 17/229
        // shape as the head burst).
        402 => {
            fx.stardust_loop(5, 3, false);
            for i in 0..fx.trail_len {
                let slot = fx.old_pos[i];
                if fx.rng.next_max(4) != 0 {
                    continue; // `:83613-83616`
                }
                if slot == Vec2::ZERO {
                    break; // `:83617-83620` — an unfilled ring slot
                }
                for _ in 0..2 {
                    if fx.rng.next_max(3) == 0 {
                        fx.dust.new_dust(
                            fx.rng, fx.ctx, fx.world, slot, fx.w, fx.h, 17, hd, -1.0,
                            0, Color::default(), 1.0,
                        ); // `:83625`
                    }
                    let i = fx.dust.new_dust(
                        fx.rng, fx.ctx, fx.world, slot, fx.w, fx.h, 229, 0.0, 0.0,
                        0, Color::default(), 1.0,
                    ); // `:83627`
                    let d = fx.dust.slot_mut(i);
                    d.no_gravity = true;
                    d.velocity *= 3.0;
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On-screen ctx: the (100,100) box sits inside the 400-px cull
    /// margin; maxDustToDraw 6000 puts the early slots in the low band
    /// (no gate draw) — the style56 witness convention.
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// The low-band accepted-spawn draw replay: pos X, pos Y, vel X,
    /// vel Y, frame row, scale (`Dust.cs:170-190`), no per-type tweak
    /// for the plain types (5/241/249/17/229/226...). Returns the six
    /// rolls.
    fn replay_spawn(r: &mut UnifiedRandom, w: i32, h: i32) -> [i32; 6] {
        [
            r.next_max(w.max(5) - 4),
            r.next_max(h.max(5) - 4),
            r.next_range(-20, 21),
            r.next_range(-20, 21),
            r.next_max(3),
            r.next_range(-20, 21),
        ]
    }

    /// `NPC.cs:82090-82093` — the DD2 Goblin trio's death half: exactly
    /// 20 type-5 dusts at `(hitDirection, -1)`, six internals each, no
    /// other draws (the five gores are constant-argument). The stream
    /// replay pins the count AND the order.
    #[test]
    fn w4_g60_the_dd2_goblin_burst_replays_20_six_draw_spawns() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(777);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 552,
            pos: Vec2::new(100.0, 100.0),
            w: 18,
            h: 38,
            vel: Vec2::new(2.0, -1.0),
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 18, 38);
        }
        assert_eq!(rng, r, "20 spawns × 6 internals, nothing else");
        for i in 0..20 {
            let d = pool.slot(i);
            assert!(d.active);
            assert_eq!(d.dust_type, 5);
        }
        assert!(!pool.slot(20).active, "exactly 20");
        // The velocity carries the recorded side: roll ×0.1 + hit_dir.
        let d0 = pool.slot(0);
        assert!(d0.velocity.x > 0.0, "hit_dir 1 rides SpeedX");
    }

    /// `NPC.cs:81894-81900` — Deerclops: 120 spawns from
    /// `position - (15,15)` with the +30 box, scale 1.5, then the
    /// Circular(4,4) velocity OVERWRITE (two NextFloat draws AFTER the
    /// six internals). The witness replays the first two iterations
    /// and pins the box/scale/velocity shape.
    #[test]
    fn w4_g60_the_deerclops_burst_replays_the_circular_overwrite() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(42);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 668,
            pos: Vec2::new(200.0, 200.0),
            w: 100,
            h: 120,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek.clone();
        for k in 0..2 {
            let rolls = replay_spawn(&mut r, 130, 150);
            // Circular: unit NextFloat then radius NextFloat.
            let a = std::f32::consts::PI * 2.0 * r.next_float();
            let rad = r.next_float();
            let vx = (a as f64).cos() as f32 * 4.0 * rad;
            let vy = (a as f64).sin() as f32 * 4.0 * rad;
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 5);
            assert_eq!(d.position.x, 185.0 + rolls[0] as f32 + 4.0);
            assert_eq!(d.position.y, 185.0 + rolls[1] as f32 + 4.0);
            assert_eq!(d.velocity, Vec2::new(vx, vy), "the overwrite, not the roll");
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.5);
        }
        // All 120 landed (the low band accepts every slot).
        assert!(pool.slot(119).active);
        assert!(!pool.slot(120).active);
        // The full-stream replay: 120 × (6 + 2).
        let mut r2 = peek;
        for _ in 0..120 {
            replay_spawn(&mut r2, 130, 150);
            r2.next_float();
            r2.next_float();
        }
        assert_eq!(rng, r2, "120 × (6 internals + 2 circular), nothing else");
    }

    /// `NPC.cs:82358-82386` — the DD2 Skeleton pair: 20×236 (six
    /// internals each — 236 hits no tweak arm) then SIX unconditional
    /// `Next(2)` gore gates consumed in order. The gore bodies are the
    /// gore lane's; this witness pins the argument-side consumption.
    #[test]
    fn w4_g60_the_dd2_skeleton_consumes_the_six_gore_gates() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 566,
            pos: Vec2::new(100.0, 100.0),
            w: 26,
            h: 44,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 26, 44);
        }
        for _ in 0..6 {
            r.next_max(2);
        }
        assert_eq!(rng, r, "20×6 internals + 6 gore gates");
        assert_eq!(pool.slot(0).dust_type, 236);
        assert!(pool.slot(19).active);
        assert!(!pool.slot(20).active);
    }

    /// `NPC.cs:82400-82423` — the DD2 Ogre pair: 20×273 then ONE
    /// `Next(5)` (the 1005/1006 head-gore pick, both facing branches).
    /// 273 is a DangerousDust id but ChildSafety defaults DISABLED
    /// (`ChildSafety.cs:33`) — no re-type draws in the default pool.
    #[test]
    fn w4_g60_the_dd2_ogre_consumes_the_head_gore_pick() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(32);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 576,
            pos: Vec2::new(100.0, 100.0),
            w: 60,
            h: 84,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 60, 84);
        }
        r.next_max(5);
        assert_eq!(rng, r, "20×6 internals + 1 head-gore pick");
        assert_eq!(pool.slot(0).dust_type, 273, "no ChildSafety re-type by default");
    }

    /// `NPC.cs:88050-88073` — the town smoke arm: 25 type-31 dusts
    /// (alpha 100, scale 2, velocity ×1.4, noLight, noGravity) then
    /// FOUR `Next(61,64)` tombstone argument draws. The 663 Princess
    /// rides THIS arm — the `:88411` second arm is dead code.
    #[test]
    fn w4_g60_the_town_smoke_consumes_the_four_tombstone_picks() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(55);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 663,
            pos: Vec2::new(100.0, 100.0),
            w: 18,
            h: 40,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..2 {
            let rolls = replay_spawn(&mut r, 18, 40);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 31);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 2.0);
            assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 * 1.4);
            assert!(d.no_light && d.no_gravity);
        }
        for _ in 2..25 {
            replay_spawn(&mut r, 18, 40);
        }
        for _ in 0..4 {
            r.next_range(61, 64);
        }
        assert_eq!(rng, r, "25×6 internals + 4 tombstone picks");
        assert!(pool.slot(24).active);
        assert!(!pool.slot(25).active);
    }

    /// `NPC.cs:86720-86728` — the Drippler: 50×5 then TWO `Next(3)`
    /// wing-gore picks (725+roll), argument-side.
    #[test]
    fn w4_g60_the_drippler_consumes_the_two_wing_picks() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 490,
            pos: Vec2::new(100.0, 100.0),
            w: 26,
            h: 26,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..50 {
            replay_spawn(&mut r, 26, 26);
        }
        r.next_max(3);
        r.next_max(3);
        assert_eq!(rng, r, "50×6 internals + 2 wing picks");
        assert!(pool.slot(49).active);
        assert!(!pool.slot(50).active);
    }

    /// `NPC.cs:84806-84809` — the Scarecrow decade: 60×194, scale 1.1
    /// (the rolled scale ×1.1, NOT an overwrite), speeds (2×hd, -2).
    #[test]
    fn w4_g60_the_scarecrow_burst_scales_the_roll() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 305,
            pos: Vec2::new(100.0, 100.0),
            w: 24,
            h: 46,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let rolls = replay_spawn(&mut r, 24, 46);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 194);
        assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.1);
        assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 + 2.0);
        assert_eq!(d.velocity.y, rolls[3] as f32 * 0.1 - 2.0);
        for _ in 1..60 {
            replay_spawn(&mut r, 24, 46);
        }
        assert_eq!(rng, r, "60×6 internals, nothing else");
        assert!(pool.slot(59).active);
        assert!(!pool.slot(60).active);
    }

    /// `NPC.cs:85002-85008` — the Dungeon Spirit death burst: 50×180
    /// at the NPC velocity, then velocity ×2 + noGravity + scale = 1.4
    /// OVERWRITE (the rolled scale is consumed but replaced). The
    /// style56 cited skip, now drawn.
    #[test]
    fn w4_g60_the_spirit_burst_overwrites_the_scale() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(81);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 288,
            pos: Vec2::new(100.0, 100.0),
            w: 20,
            h: 20,
            vel: Vec2::new(3.0, -1.0),
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let rolls = replay_spawn(&mut r, 20, 20);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 180);
        assert_eq!(d.scale, 1.4, "assignment, not the rolled scale");
        assert_eq!(d.velocity.x, (rolls[2] as f32 * 0.1 + 3.0) * 2.0);
        assert_eq!(d.velocity.y, (rolls[3] as f32 * 0.1 - 1.0) * 2.0);
        assert!(d.no_gravity);
        for _ in 1..50 {
            replay_spawn(&mut r, 20, 20);
        }
        assert_eq!(rng, r, "50×6 internals, nothing else");
        assert!(pool.slot(49).active);
        assert!(!pool.slot(50).active);
    }

    /// `NPC.cs:89065-89079` — the Fungi Spore: 20×165 alpha 50 scale
    /// 1.5, velocity ×2 + noGravity; the three gores are type!=261
    /// gated so 261 spawns NONE (no argument draws).
    #[test]
    fn w4_g60_the_fungi_spore_smoke_has_no_gore_draws() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(82);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 261,
            pos: Vec2::new(100.0, 100.0),
            w: 30,
            h: 30,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..20 {
            replay_spawn(&mut r, 30, 30);
        }
        assert_eq!(rng, r, "20×6 internals, no gore draws");
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 165);
        assert_eq!(d.alpha, 50);
        assert!(d.no_gravity);
    }

    /// `NPC.cs:87248-87257` — the Brain of Cthulhu: 150×5 then SIX
    /// gores × TWO `Next(-30,31)` velocity argument draws each.
    #[test]
    fn w4_g60_the_brain_consumes_the_twelve_gore_velocity_draws() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(91);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 266,
            pos: Vec2::new(100.0, 100.0),
            w: 100,
            h: 100,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..150 {
            replay_spawn(&mut r, 100, 100);
        }
        for _ in 0..6 {
            r.next_range(-30, 31);
            r.next_range(-30, 31);
        }
        assert_eq!(rng, r, "150×6 internals + 12 gore velocity draws");
        assert!(pool.slot(149).active);
        assert!(!pool.slot(150).active);
    }

    /// `NPC.cs:87226-87236` — the Spore: 15 × (gate + spawn); the
    /// `Next(3) != 0` gate draws EVERY iteration and picks 166 vs 167.
    #[test]
    fn w4_g60_the_spore_gate_draws_every_iteration() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(92);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 265,
            pos: Vec2::new(100.0, 100.0),
            w: 22,
            h: 22,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..15 {
            let g = r.next_max(3);
            replay_spawn(&mut r, 22, 22);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, if g != 0 { 166 } else { 167 });
        }
        assert_eq!(rng, r, "15 × (1 gate + 6 internals)");
        assert!(pool.slot(14).active);
        assert!(!pool.slot(15).active);
    }

    /// `NPC.cs:84140-84155` — the Cultist: 30×5 + 10×16 (scale 1.5,
    /// velocity ×2, noGravity), then ONE `Next(1,4)` hood count and
    /// that many `Next(11,14)` type picks.
    #[test]
    fn w4_g60_the_cultist_hood_count_gates_the_type_picks() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 439,
            pos: Vec2::new(100.0, 100.0),
            w: 30,
            h: 50,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..30 {
            replay_spawn(&mut r, 30, 50);
        }
        for k in 30..40 {
            let rolls = replay_spawn(&mut r, 30, 50);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 16);
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.5);
            assert!(d.no_gravity);
        }
        let n = r.next_range(1, 4);
        for _ in 0..n {
            r.next_range(11, 14);
        }
        assert_eq!(rng, r, "40×6 internals + 1 count + n picks");
        assert!(pool.slot(39).active);
        assert!(!pool.slot(40).active);
    }

    /// `NPC.cs:84567-84571` — the Saucer family: 30×228 with the
    /// NewDust 228 tweak (vy re-roll `Next(-10,6)×0.1`, vx ×0.3, scale
    /// ×0.7 — THREE extra internals, `Dust.cs:214-220`) then the
    /// scale = 0.5 OVERWRITE. The witness replays the 9-draw spawn.
    #[test]
    fn w4_g60_the_saucer_228_tweak_rolls_before_the_overwrite() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(111);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 392,
            pos: Vec2::new(100.0, 100.0),
            w: 80,
            h: 40,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..2 {
            let rolls = replay_spawn(&mut r, 80, 40);
            // The 228 tweak: vy = Next(-10,6)*0.1, vx *= 0.3, scale *= 0.7.
            let tweak_vy = r.next_range(-10, 6);
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 228);
            assert_eq!(d.scale, 0.5, "the overwrite replaces roll×0.7");
            assert_eq!(d.velocity.x, (rolls[2] as f32 * 0.1 - 1.5) * 0.3);
            assert_eq!(d.velocity.y, tweak_vy as f32 * 0.1, "the re-roll, not roll-1.5");
        }
        for _ in 2..30 {
            replay_spawn(&mut r, 80, 40);
            r.next_range(-10, 6);
        }
        assert_eq!(rng, r, "30 × (6 internals + 1 tweak re-roll)");
        assert!(pool.slot(29).active);
        assert!(!pool.slot(30).active);
    }

    /// `NPC.cs:83105-83118` — Vortex Alien: 10×241 then 20 ×
    /// (SelectRandom `Next(3)` pick BEFORE the spawn, then the 229/240
    /// internals, then scale `1.25 + NextFloat`). The pick precedes
    /// the spawn (argument evaluation order).
    #[test]
    fn w4_g60_the_vortex_alien_pick_precedes_the_spawn() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(121);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 427,
            pos: Vec2::new(100.0, 100.0),
            w: 34,
            h: 34,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..10 {
            replay_spawn(&mut r, 34, 34);
        }
        for k in 10..30 {
            let pick = r.next_max(3);
            let ty = [229, 229, 240][pick as usize];
            replay_spawn(&mut r, 34, 34);
            let extra = r.next_float();
            let d = pool.slot(k);
            assert_eq!(d.dust_type, ty);
            assert_eq!(d.scale, 1.25 + extra);
            assert_eq!(d.fade_in, 0.25);
            assert!(d.no_gravity && d.no_light);
        }
        assert_eq!(rng, r, "10×6 + 20×(1 pick + 6 internals + 1 scale)");
        assert!(pool.slot(29).active);
        assert!(!pool.slot(30).active);
    }

    /// `NPC.cs:82873-82880` — the Crawltipede family: 6×241 then 2×6
    /// with FULL arguments (alpha 100, Transparent, scale 2) — the 6
    /// hits the NewDust tweak (vy re-roll, vx ×0.3, scale ×0.7) but
    /// the C# passes scale 2 as the argument so the rolled×0.7×2 is
    /// the result; NO post writes. The witness pins the 6's tweak.
    #[test]
    fn w4_g60_the_crawltipede_six_dusts_carry_full_args() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(122);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 412,
            pos: Vec2::new(100.0, 100.0),
            w: 30,
            h: 30,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for _ in 0..6 {
            replay_spawn(&mut r, 30, 30);
        }
        for k in 6..8 {
            let rolls = replay_spawn(&mut r, 30, 30);
            let tweak_vy = r.next_range(-10, 6); // the type-6 tweak
            let d = pool.slot(k);
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.color, Color::TRANSPARENT);
            assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 2.0 * 0.7);
            assert_eq!(d.velocity.y, tweak_vy as f32 * 0.1);
        }
        assert_eq!(rng, r, "6×6 + 2×(6+1 tweak)");
        assert!(pool.slot(7).active);
        assert!(!pool.slot(8).active);
    }

    /// `NPC.cs:83601-83631` — the Stardust Worm Head: 5 × (gate + 17?
    /// + 229) then the oldPos loop's `Next(4)` gates until the first
    /// `== 0` breaks — an ALL-ZERO ring (a never-updated corpse; the
    /// live ring rides `Npc::update_trails`, the next witness).
    /// TRAILCACHE WIDENING: the gate walk bounds on `trail_len` (the
    /// resized `oldPos.Length`, `:83611` NO min) — up to 36 gates for
    /// 402 (`NPCID.cs:10450`), was 10.
    #[test]
    fn w4_g60_the_worm_head_trail_gates_until_the_zero_break() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 402,
            pos: Vec2::new(100.0, 100.0),
            w: 32,
            h: 32,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 36,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let mut spawned = 0usize;
        for _ in 0..5 {
            let g = r.next_max(3);
            if g == 0 {
                replay_spawn(&mut r, 32, 32);
                spawned += 1;
            }
            replay_spawn(&mut r, 32, 32); // the always-229
            spawned += 1;
        }
        // The trail gates: draw until the first == 0 (max 36 — the
        // resized length, `:83611`'s `num179 < this.oldPos.Length`).
        let mut gates = 0;
        for _ in 0..36 {
            gates += 1;
            if r.next_max(4) == 0 {
                break;
            }
        }
        assert_eq!(rng, r, "5 pair iterations + {gates} trail gates");
        assert!(pool.slot(spawned - 1).active);
        assert!(!pool.slot(spawned).active);
    }

    /// `NPC.cs:83611-83631` with a LIVE ring (the W4-g60 wiring wave):
    /// slots 0-2 carry positions, slot 3 is zero. Every slot draws its
    /// `Next(4)` gate; a `== 0` gate on a filled slot spawns TWO gated
    /// pairs AT THE SLOT (not the head position); the zero slot breaks
    /// AFTER its gate draw — slots 4-35 draw nothing (TRAILCACHE
    /// WIDENING: the 402 ring is 36 logical slots, storage 60 — the
    /// walk bounds on `trail_len`, `:83611` NO min).
    #[test]
    fn w4_g60_the_worm_head_trail_spawns_at_the_live_slots() {
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(4242);
        let peek = rng.clone();
        let mut ring = [Vec2::ZERO; 60];
        ring[0] = Vec2::new(100.0, 100.0);
        ring[1] = Vec2::new(94.0, 101.0);
        ring[2] = Vec2::new(88.0, 103.0);
        let mut fx = DeathFx {
            ty: 402,
            pos: Vec2::new(106.0, 100.0),
            w: 32,
            h: 32,
            vel: Vec2::ZERO,
            hit_dir: -1,
            old_pos: ring,
            trail_len: 36,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        let mut spawned = 0usize;
        let mut slot_positions: Vec<Vec2> = Vec::new();
        for _ in 0..5 {
            if r.next_max(3) == 0 {
                replay_spawn(&mut r, 32, 32);
                spawned += 1;
            }
            replay_spawn(&mut r, 32, 32);
            spawned += 1;
        }
        for &slot in &ring[..36] {
            if r.next_max(4) != 0 {
                continue;
            }
            if slot == Vec2::ZERO {
                break;
            }
            for _ in 0..2 {
                if r.next_max(3) == 0 {
                    replay_spawn(&mut r, 32, 32);
                    slot_positions.push(slot);
                    spawned += 1;
                }
                replay_spawn(&mut r, 32, 32);
                slot_positions.push(slot);
                spawned += 1;
            }
        }
        assert_eq!(rng, r, "the head burst + the gated trail pairs");
        // Every trail dust spawned inside its slot's box — the default
        // block stamps `position + Next(w-4) + 4` (`Dust.cs:170-171`),
        // so the bounds are `[slot+4, slot+w)` — NOT the head position
        // (106,100) when the slot differs.
        for (k, &slot) in slot_positions.iter().enumerate() {
            let d = pool.slot(spawned - slot_positions.len() + k);
            assert!(
                d.position.x >= slot.x + 4.0 && d.position.x < slot.x + 32.0,
                "trail dust {k} rides its slot's X"
            );
            assert!(
                d.position.y >= slot.y + 4.0 && d.position.y < slot.y + 32.0,
                "trail dust {k} rides its slot's Y"
            );
        }
        assert!(pool.slot(spawned - 1).active);
        assert!(!pool.slot(spawned).active);
    }

    /// The lane gate: 388/418 stay c12's, uncovered kinds no-op with
    /// ZERO draws (the wrapper's early return).
    #[test]
    fn w4_g60_uncovered_kinds_draw_nothing() {
        assert!(lane_covers(668));
        assert!(lane_covers(552));
        assert!(!lane_covers(388), "c12's feed");
        assert!(!lane_covers(418), "c12's feed");
        assert!(!lane_covers(1), "g59's half");
        assert!(!lane_covers(400), "no arm");
        let w = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 400,
            pos: Vec2::new(100.0, 100.0),
            w: 20,
            h: 20,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &w,
        };
        death_dust_draws(&mut fx);
        assert_eq!(rng, peek, "the default arm draws nothing");
        assert!(!pool.slot(0).active);
    }

    /// The gore-remainder sweep (§11, f5-owned arms): every listed kind
    /// pays EXACTLY its census dust count on the UpdateNPCs stream and
    /// ZERO gore-argument draws — all gores in these arms are
    /// constant-argument (the shimmer/altTexture picks are data). The
    /// per-kind draw counts pin the C# death halves one witness row
    /// each; the shared-arm kinds ride their own row.
    #[test]
    fn w4_g60_gore_tail_f5_arms_pay_dust_only() {
        // (kind, dust count, dust type, w, h, seed) — the §11 census.
        for (ty, n, dty, w, h, seed) in [
            (250u16, 20i32, 54i32, 30i32, 30i32, 201i32), // :88971 3×99
            (315, 20, 54, 30, 40, 202),                  // :89038 3×99+446
            (329, 50, 5, 26, 40, 203),                   // :87745 485-487
            (341, 20, 196, 30, 30, 205),                 // :88755 61/62/63
            (353, 50, 5, 20, 40, 207),                   // :88208 6 cond.
            (354, 50, 5, 20, 40, 208),                   // the shared arm
            (368, 50, 5, 20, 40, 209),                   // :88033 5 cond.
            (441, 50, 5, 20, 40, 210),                   // :87685 768-771
            (471, 50, 5, 30, 46, 211),                   // :89617 675-678
            (550, 50, 5, 20, 40, 213),                   // :87952 967-969
            (588, 50, 5, 20, 40, 214),                   // :87972 5 cond.
            (589, 50, 5, 20, 40, 215),                   // the shared arm
            (633, 50, 5, 20, 40, 216),                   // :87995 6 picks
            (267, 20, 5, 30, 30, 217),                   // :87269 gore 402
            (338, 50, 5, 20, 40, 218),                   // :86878 494+tail
            (339, 50, 5, 20, 40, 219),                   // 495 head
            (340, 50, 5, 20, 40, 220),                   // 496 head
            (342, 50, 0, 20, 40, 221),                   // :86858 500-502
            (343, 50, 5, 26, 46, 222),                   // :86908 503-506
            (348, 10, 5, 24, 40, 223),                   // :86322 528-532
            (349, 10, 5, 24, 40, 224),                   // the shared arm
            (350, 10, 5, 20, 40, 225),                   // :86362 533-536
            (351, 10, 5, 24, 40, 226),                   // :86342 538-542
            (489, 50, 5, 20, 40, 227),                   // :86669 722-724
        ] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w,
                h,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 60],
                trail_len: 10,
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..n {
                let rolls = replay_spawn(&mut r, w, h);
                let d = pool.slot(k as usize);
                assert_eq!(d.dust_type, dty, "{ty} dust {k}");
                if dty == 54 {
                    // the smoke54 full-arg shape: alpha 50, scale roll
                    // ×1.5 then velocity ×2 + noGravity (`:89039-89042`).
                    assert_eq!(d.alpha, 50);
                    assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.5);
                    assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 * 2.0);
                    assert!(d.no_gravity);
                }
                if ty == 341 {
                    // the Present Mimic's plain (0,0) burst — no writes.
                    assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1);
                }
            }
            assert_eq!(rng, r, "{ty}: {n}×6 internals, zero gore draws");
            assert!(pool.slot((n - 1) as usize).active);
            assert!(!pool.slot(n as usize).active, "{ty}: exactly {n} dusts");
        }
    }

    /// The gore-remainder sweep (§11, f10-owned arms): the martian
    /// machines + probe + walker + spider-small. The 226 bursts carry
    /// the scale-0.5 OVERWRITE (no extra internals); 227 is a plain
    /// 6-draw spawn; the 410 stardust pair is 5×(17+229) ungated. All
    /// gores constant-argument (the probe's ai[0]==3 velocity is data).
    #[test]
    fn w4_g60_gore_tail_f10_arms_pay_dust_only() {
        for (ty, n, w, h, seed) in [
            (385u16, 30i32, 24i32, 40i32, 302i32), // :84373 597-599
            (386, 30, 24, 40, 303),                // :84393 600-602
            (387, 30, 30, 30, 304),                // :84415 603×2/604
            (389, 30, 30, 40, 305),                // :84459 904-906
            (391, 30, 40, 20, 306),                // :84545 607/608/609
            (399, 10, 16, 16, 307),                // :84515 898/899
            (520, 30, 36, 56, 308),                // :84438 851/852+3
            (393, 30, 40, 40, 310),                // :84567 gore 616
            (394, 30, 40, 40, 311),                // :84567 gore 617
            (395, 30, 60, 60, 312),                // :84567 610-615
            (306, 60, 24, 46, 313),                // :84806 441-445
            (307, 60, 24, 46, 314),                // the shared arm
            (308, 60, 24, 46, 315),                // the shared arm
            (309, 60, 24, 46, 316),                // the shared arm
            (310, 60, 24, 46, 317),                // the shared arm
            (311, 60, 24, 46, 318),                // the shared arm
            (312, 60, 24, 46, 319),                // the shared arm
            (313, 60, 24, 46, 320),                // the shared arm
            (314, 60, 24, 46, 321),                // the shared arm
            (325, 60, 30, 50, 322),                // :84849 469-475
            (327, 60, 40, 40, 323),                // :84828 476-484
            (344, 60, 40, 60, 324),                // :84878 508-512×2
        ] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w,
                h,
                vel: Vec2::ZERO,
                hit_dir: -1,
                old_pos: [Vec2::ZERO; 60],
                trail_len: 10,
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..n {
                let rolls = replay_spawn(&mut r, w, h);
                let d = pool.slot(k as usize);
                match ty {
                    385 | 386 | 389 | 391 => assert_eq!(d.dust_type, 227),
                    393..=395 => {
                        // the 228 tweak re-rolls vy (one extra draw),
                        // then the scale-0.5 overwrite (`:84569-84570`).
                        assert_eq!(d.dust_type, 228);
                        r.next_range(-10, 6);
                        assert_eq!(d.scale, 0.5);
                    }
                    306..=314 | 325 | 327 => {
                        let dty = match ty {
                            325 => 191,
                            327 => 189,
                            _ => 194,
                        };
                        assert_eq!(d.dust_type, dty);
                        assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.1);
                    }
                    344 => assert_eq!(d.dust_type, 196),
                    _ => {
                        assert_eq!(d.dust_type, 226);
                        assert_eq!(d.scale, 0.5, "the 226 overwrite");
                        let k_f = if ty == 399 { 2.5 } else { 1.5 };
                        // hit_dir = -1 → sx = -k×hd = +k
                        assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 + k_f);
                    }
                }
            }
            assert_eq!(rng, r, "{ty}: {n}×6 internals, zero gore draws");
            assert!(pool.slot((n - 1) as usize).active);
            assert!(!pool.slot(n as usize).active, "{ty}: exactly {n} dusts");
        }
        // 410 Stardust Spider (small): 5 × (17 + 229 noGravity ×3), no
        // gates on the death half (`NPC.cs:83523-83529`); no gores.
        let wld = air();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(309);
        let peek = rng.clone();
        let mut fx = DeathFx {
            ty: 410,
            pos: Vec2::new(100.0, 100.0),
            w: 16,
            h: 16,
            vel: Vec2::ZERO,
            hit_dir: 1,
            old_pos: [Vec2::ZERO; 60],
            trail_len: 10,
            rng: &mut rng,
            dust: &mut pool,
            ctx: &c,
            world: &wld,
        };
        death_dust_draws(&mut fx);
        let mut r = peek;
        for k in 0..5 {
            replay_spawn(&mut r, 16, 16);
            assert_eq!(pool.slot(k * 2).dust_type, 17);
            replay_spawn(&mut r, 16, 16);
            let d = pool.slot(k * 2 + 1);
            assert_eq!(d.dust_type, 229);
            assert!(d.no_gravity);
        }
        assert_eq!(rng, r, "410: 5×(17+229) pairs, zero gore draws");
        assert!(pool.slot(9).active);
        assert!(!pool.slot(10).active);
    }

    /// `NPC.cs:87186-87218` — the Plantera family, per kind: 150×167
    /// (the 265 short-circuit kills the gate draw for ALL three family
    /// types), then the per-kind gore-position pairs: 262 pays NINE
    /// `Next(w)+Next(h)` pairs (gores 381-387 with 386/387 doubled),
    /// 263/264 pay TWO each (390/391, 388/389). The gores are f5's
    /// (claimed `97c70c97`); g60 consumes exactly 18/4/4 argument draws.
    #[test]
    fn w4_g60_gore_tail_plantera_trio_pays_per_kind_position_pairs() {
        for (ty, n_gores, seed) in [(262u16, 9i32, 501i32), (263, 2, 502), (264, 2, 503)] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w: 48,
                h: 48,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 60],
                trail_len: 10,
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..150 {
                replay_spawn(&mut r, 48, 48);
                assert_eq!(pool.slot(k).dust_type, 167, "{ty}: no 166 gate");
            }
            for _ in 0..n_gores {
                r.next_max(48);
                r.next_max(48);
            }
            assert_eq!(rng, r, "{ty}: 150×6 + {} pairs", n_gores * 2);
            assert!(pool.slot(149).active);
            assert!(!pool.slot(150).active, "{ty}: exactly 150 dusts");
        }
    }

    /// `NPC.cs:88048-88073` — the town smoke arm's OTHER two g60 kinds
    /// (369 Angler / 376 Sleeping Angler; 663 rides the dedicated
    /// tombstone witness): each pays 25×31 (alpha 100, scale ×2 roll,
    /// velocity ×1.4, noLight+noGravity) + FOUR `Next(61,64)` tombstone
    /// picks. The gores (61/62/63 with scale-1 + velocity±1 writes) are
    /// f5's — claimed, not yet pushed (`:88046` is past f5's `:86986`).
    #[test]
    fn w4_g60_gore_tail_angler_pair_pays_the_four_tombstone_picks() {
        for (ty, seed) in [(369u16, 601i32), (376, 602)] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w: 18,
                h: 40,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 60],
                trail_len: 10,
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..25 {
                let rolls = replay_spawn(&mut r, 18, 40);
                let d = pool.slot(k);
                assert_eq!(d.dust_type, 31);
                assert_eq!(d.alpha, 100);
                assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 2.0);
                assert_eq!(d.velocity.x, rolls[2] as f32 * 0.1 * 1.4);
                assert!(d.no_light && d.no_gravity);
            }
            for _ in 0..4 {
                r.next_range(61, 64);
            }
            assert_eq!(rng, r, "{ty}: 25×6 internals + 4 tombstone picks");
            assert!(pool.slot(24).active);
            assert!(!pool.slot(25).active, "{ty}: exactly 25 dusts");
        }
    }

    /// The gore-remainder sweep (§11, the NO-gore arms): 330/352/384/
    /// 533 have zero `NewGore` sites — their whole death half is the
    /// dust burst (288/265/402 already carry dedicated witnesses; 326
    /// shares the Mourning Wood shape, 663 folds onto the town smoke).
    #[test]
    fn w4_g60_gore_tail_no_gore_arms_pay_dust_only() {
        // (kind, n, dust type, w, h, seed)
        for (ty, n, dty, w, h, seed) in [
            (326u16, 60i32, 191i32, 20i32, 20i32, 401i32), // :84903
            (330, 50, 192, 30, 40, 402),                   // :88996
            (352, 60, 67, 20, 20, 403),                    // :84788
            (384, 70, 226, 20, 20, 404),                   // :84355
            (533, 40, 27, 30, 46, 405),                    // :83880
        ] {
            let wld = air();
            let c = ctx();
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            let peek = rng.clone();
            let mut fx = DeathFx {
                ty,
                pos: Vec2::new(100.0, 100.0),
                w,
                h,
                vel: Vec2::ZERO,
                hit_dir: 1,
                old_pos: [Vec2::ZERO; 60],
                trail_len: 10,
                rng: &mut rng,
                dust: &mut pool,
                ctx: &c,
                world: &wld,
            };
            death_dust_draws(&mut fx);
            let mut r = peek;
            for k in 0..n {
                let rolls = replay_spawn(&mut r, w, h);
                let d = pool.slot(k as usize);
                assert_eq!(d.dust_type, dty, "{ty} dust {k}");
                match ty {
                    326 => assert_eq!(d.scale, (1.0 + rolls[5] as f32 * 0.01) * 1.1),
                    330 => {
                        assert_eq!(d.alpha, 100);
                        assert!(d.no_gravity);
                    }
                    352 => assert!(d.no_gravity),
                    384 => assert_eq!(d.scale, 0.5),
                    533 => {
                        assert_eq!(d.alpha, 50);
                        assert!(d.no_gravity);
                        assert_eq!(d.fade_in, 1.0);
                    }
                    _ => unreachable!(),
                }
            }
            assert_eq!(rng, r, "{ty}: {n}×6 internals, zero gore draws");
            assert!(pool.slot((n - 1) as usize).active);
            assert!(!pool.slot(n as usize).active, "{ty}: exactly {n} dusts");
        }
    }
}

/// The batch-1 death dust feed: draws the kind's `HitEffect` DEATH half in
/// C# order. Returns true when an arm ran (the wiring gate mirrors the
/// match; the flag is the witness seam). `npc_type` is the C# `NPC.type`
/// (`NpcKind::cs_id`).
pub fn death_dust(
    npc: &Npc,
    npc_type: i32,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) -> bool {
    let (w, h) = npc.size();
    let (w, h) = (w as i32, h as i32);
    let pos = Vec2::new(npc.x, npc.y);
    let dir = npc.last_hit_dir as f32;
    match npc_type {
        // `NPC.cs:89487-89501` — the Meteor Head: 50 Next(2)-selected
        // 25/6 puffs, then 50 type-6 embers at velocity×0.2, scale 2.5,
        // velocity×6 + noGravity.
        23 => {
            for _ in 0..50 {
                let mut ty = 25;
                if rng.next_max(2) == 0 {
                    ty = 6;
                }
                dust.new_dust(
                    rng, ctx, world, pos, w, h, ty, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 6,
                    npc.vx * 0.2, npc.vy * 0.2, 100, Color::default(), 2.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 6.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:85490-85494` — the 120/137/138 arm's death half: 50
        // type-71 puffs, alpha 200, velocity×1.5.
        120 | 137 | 138 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 71, dir, 0.0, 200,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 1.5;
            }
        }
        // `NPC.cs:85370-85375` — the snowman trio: 50 type-76 puffs,
        // noGravity, scale×1.2 (the SpeedY stays -1, the hit half's value).
        143 | 144 | 145 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 76, dir, -1.0, 0,
                    Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.2;
            }
        }
        // `NPC.cs:85350-85355` — the 147/184 pair: 30 × (type-76
        // noGravity + type-4 tinted) pairs, SpeedY -2 on both.
        147 | 184 => {
            for _ in 0..30 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 76, dir, -2.0, 0,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).no_gravity = true;
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, dir, -2.0, 100,
                    Color::new(97, 200, 255, 100), 1.0,
                );
            }
        }
        // `NPC.cs:85420-85424` — the Crimslime: 50 type-136 puffs at
        // `this.alpha`/`this.color` (the color stand-in: default, see the
        // module doc), velocity×2.
        183 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 136, dir, 0.0,
                    npc.alpha as i32, Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 2.0;
            }
        }
        // `NPC.cs:85065-85077` — the bees' cond-death arm: 6 type-150
        // puffs; the Next(2) gate picks noGravity+scale 1.5 vs scale 0.8.
        210 | 211 => {
            for _ in 0..6 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 150, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
                let d = dust.slot_mut(idx);
                if rng.next_max(2) == 0 {
                    d.no_gravity = true;
                    d.scale = 1.5;
                } else {
                    d.scale = 0.8;
                }
            }
        }
        // ---- batch 2 (the gore-tail kinds; the dust is the whole feed —
        // every gore type below is a constant / `this.type` arithmetic, so
        // the tails draw NO arguments; the NewGore bodies + kicks are d55's).
        // `NPC.cs:86459-86462` — the Demon Eye: 50 type-5 puffs.
        2 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87351-87354` — the Servant of Cthulhu: 20 type-5 puffs.
        5 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87512-87515` / `:87566-87569` / `:87632-87635` — the
        // Eater of Souls (6), Devourer head (7) and EoW head (13): 50
        // type-18 puffs at `this.alpha`/`this.color`/`this.scale`. The
        // color stand-in is default (none of the three writes a color);
        // the alpha reads the LIVE field — the EoW's 255 spawn fade is
        // ported (the ctor row + the worm pass's `:52251-52271` tail,
        // the DOC-SWEEP), 6/7 keep the ctor 0. The 8/9 Devourer and
        // 14/15 EoW Hide-fold segments ride the heads' cs_ids (the
        // base's bestiary fold — the feed sees 7/13; the C# `:87556`/
        // `:87622` arms cover the segments with the same burst; batch
        // 16 records the folds).
        6 | 7 | 13 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:87600-87603` — the Giant Worm head: 10 type-5 puffs at
        // 2.5×hitDirection, -2.5. The 11/12 Hide-fold segments ride the
        // head's cs_id 10 (the base's bestiary fold — the C# `:87590`
        // arm covers 10-12 with the same burst; batch 16 records the
        // fold).
        10 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87662-87665` / `:88292-88295` / `:88315-88318` /
        // `:88358-88361` — the Merchant/Nurse/Arms Dealer/Dryad: 50 type-5
        // puffs at 2.5×hitDirection, -2.5 (the shimmer/altTexture gore
        // gates draw nothing).
        17 | 18 | 19 | 20 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 3 (the skeleton/goblin/Skeletron families).
        // `NPC.cs:88537-88540` — the Skeleton (21) / Dark Caster (32): 20
        // type-26 bone puffs at 2.5×dir/-2.5 (the big skeleton arm's dust
        // head; the gore tail's types are constants, no argument draws).
        // 31 AngryBones rides the SAME arm (batch 11, the d34 row landing —
        // `5f3435a8`'s hand row; its gore path falls through the ladder's
        // per-kind gates to the generic 42/43/43/44/44 tail `:88623-88639`,
        // all constants + `this.scale`).
        21 | 31 | 32 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87932-87935` — the Guide: 50 type-5 puffs at
        // 2.5×dir/-2.5 (the shimmer gore gates are field reads).
        22 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89514-89519` — the Fire Imp: 50 type-6 embers at the
        // corpse's velocity, scale 2.5, noGravity + velocity×2.
        24 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 6, npc.vx, npc.vy, 100,
                    Color::default(), 2.5,
                );
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        // `NPC.cs:89560-89563` — the goblin quartet (26/27/28/29): 50
        // type-5 puffs at 2.5×dir/-2.5 (the per-kind gore-type ladder
        // `:89564-89596` is constant data, no argument draws).
        26 | 27 | 28 | 29 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89177-89180` — the Skeletron head (35) / hand (36): 150
        // type-26 bone puffs at 2.5×dir/-2.5 (the RedHat gore gate
        // `:89181-89208` reads ai[3]/localAI[3], no argument draws).
        35 | 36 => {
            for _ in 0..150 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 4 (the bat/shark/piranha families + the hornet).
        // `NPC.cs:88335-88338` / `:88252-88255` — the Demolitionist (38) /
        // Clothier (54): 50 type-5 puffs at 2.5×dir/-2.5.
        38 | 54 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89712-89715` — the Hornet (42): 50 type-18 puffs at
        // `this.alpha`/`this.color`/`this.scale`, SpeedX = dir (the 176/
        // 205/231-235 siblings are other lanes' kinds).
        42 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:88537-88540` — the Undead Miner (44) / Tim (45): 20
        // type-26 bone puffs (the big skeleton arm, batch 3's shape).
        44 | 45 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85807-85810` — the bat family's dust head: 15 type-5
        // puffs at 2×dir/-2 — 49 CaveBat / 51 JungleBat (batch 4), 226
        // FlyingSnake (batch 12, d37's row), 150 IceBat / 152
        // GiantFlyingFox (batch 14, g8's rows — the shared `:85797`
        // 49/51/93/150/152/226 arm; 93 wired batch 9). The per-kind gore
        // tails `:85811-85840` (83, 107, 164/165, 175/176/176,
        // 317/318/318/319, 82) are constant types, no argument draws.
        49 | 51 | 150 | 152 | 226 => {
            for _ in 0..15 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86381-86384` — the Piranha (58) / Corrupt Bunny (47) /
        // Corrupt Goldfish (57): 10 type-5 puffs at 2×dir/-2 (the shared
        // `:86371` arm; 47/57 ride the d34 rows — batches 11/12, the
        // 78/79 + 84 gore tails `:86385-86407` are constant types, no
        // argument draws; the 464/465 siblings are g60's band).
        47 | 57 | 58 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86740-86743` — the Shark (65): 75 type-5 puffs at
        // 2×dir/-2 (the 542-545/692 siblings are other lanes' kinds).
        65 => {
            for _ in 0..75 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89803-89806` — the Voodoo Demon (66) / Demon (62): 50
        // type-5 puffs at 2×dir/-2 (the shared `:89793` arm; 62 rides the
        // d34 row — batch 12; the 93/94/94 gore tail `:89807-89809` is
        // constant types, no argument draws).
        62 | 66 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 5 (the wyvern/eye/town families + the leech).
        // `NPC.cs:88921-88932` — the Wyvern head (87): 10 type-16 puffs
        // (scale 1.5, vel×2, noGravity), then the gore tail's ARGUMENT
        // draws — Next(1,4) count + Next(11,14) per gore (the bodies +
        // kicks are d55's; the g63 claim's gore-sibling rule). The 88-92
        // Hide-fold segments ride the head's cs_id 87 (the base's
        // bestiary fold — the feed sees 87, the C# `:88909` arm covers
        // 87-92 with the same burst; batch 16 records the fold).
        87 => {
            for _ in 0..10 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 16, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            let count = rng.next_range(1, 4);
            for _ in 0..count {
                let _gore_type = rng.next_range(11, 14);
            }
        }
        // `NPC.cs:87880-87883` — the Bound Wizard (106) / Goblin Tinkerer
        // (107) / Wizard (108): 50 type-5 puffs at 2.5×dir/-2.5 (the
        // shimmer/altTexture gore gates are field reads). 105 BoundGoblin
        // rides the SAME arm (batch 16, the base-fold activation — the
        // shared `:87870` 105-108 arm; the port's `entry_for_npc` folds
        // 105→107 so the feed sees 107, the C# arm's 105/107 gore gate
        // `:87884-87891` is a field read + constants either way).
        105 | 106 | 107 | 108 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89560-89563` — the Goblin Archer (111): 50 type-5 puffs
        // (the goblin arm, batch 3's shape; the 111 gore ladder adds the
        // 131 bone, constant data).
        111 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87446-87449` — the Leech head (117): 10 type-5 puffs at
        // dir/-1 (the SpeedY stays -1, NOT -2 — the leech arm's own
        // value; the 118/119 siblings fold onto the head's bestiary id).
        117 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, dir, -1.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87912-87915` — the Bound Mechanic (123) / Mechanic
        // (124): 50 type-5 puffs at 2.5×dir/-2.5.
        123 | 124 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86459-86462` — the Cataract Eye (190) / Sleepy Eye
        // (191): 50 type-5 puffs at 2×dir/-2 (the demon-eye arm, batch
        // 2's shape; the per-kind gore types are constants).
        190 | 191 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 6 (the eye trio, the dye/party girls, the pirates).
        // `NPC.cs:86459-86462` — the Dialated/Green/Purple Eyes
        // (192/193/194): 50 type-5 puffs at 2×dir/-2 (the demon-eye arm).
        192 | 193 | 194 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88378-88381` / `:88401-88404` — the Dye Trader (207) /
        // Party Girl (208): 50 type-5 puffs at 2.5×dir/-2.5.
        207 | 208 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88458-88461` — the pirate family (212-216): 50 type-5
        // puffs at 2.5×dir/-2.5 (the per-kind gore ladder `:88462-88501`
        // is constant data).
        212 | 213 | 214 | 215 | 216 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 7 (the Queen Bee, the umbrella slime, the painter/
        // witch doctor — the final present-clean kinds).
        // `NPC.cs:87491-87494` — the Queen Bee (222): 50 type-18 puffs at
        // `this.alpha`/`this.color`/`this.scale`, SpeedX = dir (the
        // 6-gore tail `:87495-87500` is constant types).
        222 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:85038-85041` — the Umbrella Slime (225): 50 type-4
        // puffs at 2×dir/-2, alpha 100, tinted (0,80,255,100) (the 314
        // gore is a constant type).
        225 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, 2.0 * dir, -2.0, 100,
                    Color::new(0, 80, 255, 100), 1.0,
                );
            }
        }
        // `NPC.cs:88086-88089` / `:88109-88112` — the Painter (227) /
        // Witch Doctor (228): 50 type-5 puffs at 2.5×dir/-2.5 (the
        // shimmer/altTexture gore gates are field reads).
        227 | 228 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 8 (the d35 row-gated activations: the goblin scout /
        // pixie / skeleton pair / mummy trio / wraith / cursed pair).
        // `NPC.cs:89560-89563` — the Goblin Scout (73): 50 type-5 puffs at
        // 2.5×dir/-2.5 (the goblin arm, batch 3's shape; the 73 gore ladder
        // `:89579-89584` is the 629/630/631 constants, no argument draws).
        73 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85527-85531` — the Pixie (75): 50 type-55 puffs at
        // dir/0, alpha 200, `this.color` (the stand-in: default — the
        // SetDefaults reset `:8526` is the only write for 75, the paint
        // ladder `:60577-60692` is type-1-only), velocity×2. Dust-only.
        75 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 55, dir, 0.0, 200,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 2.0;
            }
        }
        // `NPC.cs:88537-88540` — the Armored Skeleton (77) / Skeleton
        // Archer (110): 20 type-26 bone puffs at 2.5×dir/-2.5 (the big
        // skeleton arm, batch 3's shape; the 77/110 gore ladders
        // `:88624-88637` add the 106/130/131 constants, no argument draws).
        77 | 110 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88946-88951` — the Mummy trio (78/79/80): 20 type-31
        // puffs at (0,0), scale 1.5, velocity×2 + noGravity (the 61/62/63
        // gore tail `:88952-88957` is constant types + hitDirection, no
        // argument draws).
        78 | 79 | 80 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 31, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:88971-88976` — the Wraith (82): 20 type-54 puffs at
        // (0,0), alpha 50, scale 1.5, velocity×2 + noGravity (the 99×3
        // gore tail `:88977-88982` is constant + hitDirection, no draws).
        // 158 VampireBat rides the SAME arm (batch 15, g8's row — the
        // shared `:88959` 82/158/159/250/253 arm; 159 stays row-gated
        // (d36's band), 250/253 are g60's).
        82 | 158 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 54, 0.0, 0.0, 50,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:87152-87163` — the Cursed Hammer (83) / Enchanted Sword
        // (84): 20 type-31 puffs at (0,0), scale 1.5, velocity×2 +
        // noGravity, THEN the gore tail's ARGUMENT draws — 3 gores each
        // with a `Next(-2,3)` x/y velocity pair (`:87158`/`:87160`/
        // `:87162`, 6 draws). The bodies + kicks stay d55's (the g63
        // gore-sibling rule); the draws are consumed here in C# order.
        // 179 CrimsonAxe rides the SAME arm (batch 15, g8's row — the
        // shared `:87141` 83/84/179 arm, the same 6 argument draws).
        83 | 84 | 179 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 31, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            for _ in 0..3 {
                let _gx = rng.next_range(-2, 3);
                let _gy = rng.next_range(-2, 3);
            }
        }
        // ---- batch 9 (the d35 row-gated activations: the mimic / unicorn /
        // giant bat / corruptor / digger trio / seeker trio).
        // `NPC.cs:88785-88788` — the Mimic (85): 20 puffs of the ai[3]
        // selector type (`:88768-88776`: 2→10, 3→37, else 7), default
        // args. The 61/62/63 gore tail `:88789-88794` is constant types +
        // hitDirection (no argument draws).
        85 => {
            let mut ty = 7;
            if npc.ai[3] == 2.0 {
                ty = 10;
            }
            if npc.ai[3] == 3.0 {
                ty = 37;
            }
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, ty, 0.0, 0.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87705-87708` — the Unicorn (86): 50 type-5 puffs at
        // 2.5×dir/-2.5 (the 101-105 gore tail `:87709-87715` is constant
        // types, no argument draws).
        86 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85807-85810` — the Giant Bat (93): 15 type-5 puffs at
        // 2×dir/-2 (the bat arm, batch 4's shape; the 107 gore
        // `:85815-85818` is a constant type, no argument draws).
        93 => {
            for _ in 0..15 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87512-87515` — the Corruptor (94): 50 type-18 puffs at
        // dir/-2, `this.alpha`/`this.color`/`this.scale` (the EoSouls arm,
        // batch 2's shape; the color stand-in is default — 94 writes no
        // color). The 108/108/109/110 gore tail `:87516-87522` is constant
        // types, no argument draws.
        94 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:87616-87619` — the Digger head (95) / body (96) / tail
        // (97): 10 type-5 puffs at 2.5×dir/-2.5 (the Giant Worm arm,
        // batch 2's shape; the `this.type - 95 + 111` gore `:87620` is
        // constant arithmetic, no argument draws).
        95 | 96 | 97 => {
            for _ in 0..10 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:87583-87586` — the Seeker head (98) / body (99) / tail
        // (100): 50 type-18 puffs at dir/-2, `this.alpha`/`this.color`/
        // `this.scale` (the EoSouls arm; the color stand-in is default —
        // none writes a color). The 110 gore `:87587-87588` is a constant
        // type + `this.alpha` post-write, no argument draws.
        98 | 99 | 100 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // ---- batch 10 (the d35 row-gated activations: the clinger /
        // gastropod / possessed armor — the lane's final d35 kinds).
        // `NPC.cs:87547-87550` — the Clinger (101): 50 type-18 puffs at
        // dir/-2, `this.alpha`/`this.color`/`this.scale` (the EoSouls arm;
        // the color stand-in is default — 101 writes no color). The
        // 110/114/114/115 gore tail `:87551-87554` is constant types, no
        // argument draws.
        101 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // `NPC.cs:85554-85557` — the Green Jellyfish (103): 25 type-4
        // puffs at 2×dir/-2, alpha 0, tinted (70,225,140,100) (the shared
        // 63/64/103 arm `:85534-85558`'s 103 color row `:85541-85544`;
        // 63/64 stay row-gated — their kinds are absent, the 50,120,255 /
        // 225,70,140 tints land with them). Dust-only, zero gore. The
        // census's `:85541` color-only nested row is a selection, not a
        // second death arm (the g59_order dust>0 artifact).
        103 => {
            for _ in 0..25 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, 2.0 * dir, -2.0, 0,
                    Color::new(70, 225, 140, 100), 1.0,
                );
            }
        }
        // `NPC.cs:85509-85513` — the Gastropod (122): 50 type-72 puffs at
        // dir/0, alpha 200, velocity×1.5 (the Illuminant arm's shape with
        // type 72, NOT 71 — the gastropod's own row). Dust-only, zero gore.
        122 => {
            for _ in 0..50 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 72, dir, 0.0, 200,
                    Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity *= 1.5;
            }
        }
        // `NPC.cs:89085-89090` — the Possessed Armor (140): 20 type-54
        // puffs at (0,0), alpha 50, scale 1.5, velocity×2 + noGravity (the
        // Wraith arm's shape, a DEATH-ONLY `life <= 0` gate with no hit
        // half). The 99×3 gore tail `:89091-89096` is constant +
        // hitDirection, no argument draws.
        140 => {
            for _ in 0..20 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 54, 0.0, 0.0, 50,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // ---- batch 11 (the d34 row-gated activations: the bone/caster/
        // man-eater families). d34's pushed tip `5f3435a8` (phases 1-3)
        // landed the cs_id hand rows 31/34/37/39/43/47/48/56 (+ the 40/41
        // Hide-folds onto 39). 31/47 ride the extended shared arms above;
        // the six new arms follow.
        // `NPC.cs:89786-89789` — the Harpy (48): 50 type-5 puffs at
        // 2×dir/-2 (the 80/81 gore tail `:89790-89791` is constant types,
        // no argument draws).
        48 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89132-89141` — the Cursed Skull (34): 15 × (type-15
        // noGravity/noLight alpha 100 scale 1.8 + type-26 noLight scale
        // 0.9) pairs at -velocity×0.2, each velocity×1.3. Dust-only,
        // zero gore (the arm's hit half `:89119-89128` is g62's).
        34 => {
            for _ in 0..15 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 15,
                    -npc.vx * 0.2, -npc.vy * 0.2, 100, Color::default(), 1.8,
                );
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.no_gravity = true;
                d.velocity *= 1.3;
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 26,
                    -npc.vx * 0.2, -npc.vy * 0.2, 0, Color::default(), 0.9,
                );
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.velocity *= 1.3;
            }
        }
        // `NPC.cs:88232-88235` — the Old Man (37): 50 type-5 puffs at
        // 2.5×dir/-2.5 (the town shape; the shimmer-gated 58-60/1328-1330
        // gore tail `:88236-88240` reads `IsShimmerVariant`, a field read
        // — no argument draws).
        37 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89109-89112` — the Bone Serpent head (39): 20 type-26
        // bone puffs at 2.5×dir/-2.5 (the skeleton shape; the
        // `this.type - 39 + 67` gore `:89113` is constant arithmetic, no
        // argument draws). The 40/41 Hide-fold segments ride the head's
        // cs_id (d34's fold — the feed sees 39; the C# arm reads the RAW
        // type, all three draw the same burst).
        39 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89752-89755` — the Man Eater (43) / Snatcher (56): 50
        // type-40 puffs at dir/-2, `this.alpha`/`this.color`, scale 1.2
        // (the color stand-in is default — neither writes a color). The
        // 72/72 gore tail `:89756-89757` is constant types, no argument
        // draws.
        43 | 56 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 40, dir, -2.0,
                    npc.alpha as i32, Color::default(), 1.2,
                );
            }
        }
        // ---- batch 12 (the d34 tail + c4/d23/d37 row-gated activations).
        // d34's 57/61/62/63, c4 `22cec16f`'s Arapaima 157, d23 `19109424`'s
        // AngryTrapper 175, d37 `750d8e44`'s beetle trio 217/218/219 +
        // FlyingSnake 226. 57/62 ride the extended shared arms above.
        // `NPC.cs:86617-86620` — the Vulture (61): 50 type-5 puffs at
        // 2×dir/-2 (the 86/87/88 gore tail `:86621-86623` is constant
        // types + a +14 X offset, no argument draws).
        61 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85554-85557` — the Blue Jellyfish (63): 25 type-4 puffs
        // at 2×dir/-2, alpha 0, tinted (50,120,255,100) (the shared
        // 63/64/103 arm's DEFAULT color row `:85536` — 63 rides the d34
        // row; 64 stays row-gated, its 225,70,140 tint lands with it; 103
        // wired in batch 10). Dust-only, zero gore.
        63 => {
            for _ in 0..25 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 4, 2.0 * dir, -2.0, 0,
                    Color::new(50, 120, 255, 100), 1.0,
                );
            }
        }
        // `NPC.cs:86514-86517` — the Arapaima (157): 50 type-5 puffs at
        // 2×dir/-2 (c4's style-44 row; the 186/187 gore tail
        // `:86518-86519` is constant types + a +14 X offset, no argument
        // draws).
        157 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:89769-89772` — the Angry Trapper (175): 50 type-40 puffs
        // at dir/-2, `this.alpha`/`this.color`, scale 1.2 (the man-eater
        // arm's shape, its OWN C# arm; d23's style-13 row). The 227/228
        // gore tail `:89773-89774` is constant types, no argument draws.
        175 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 40, dir, -2.0,
                    npc.alpha as i32, Color::default(), 1.2,
                );
            }
        }
        // `NPC.cs:87462-87465` — the beetle trio (217 Cochineal / 218 Cyan
        // / 219 Lac): 50 type-18 puffs at dir/-2, `this.alpha`/`this.color`
        // /`this.scale` (the EoSouls arm; d37's rows). The per-kind gore
        // ladder `:87466-87479` (292/293, 295/296, 294) is constant types
        // + `this.scale`, no argument draws.
        217 | 218 | 219 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, dir, -2.0,
                    npc.alpha as i32, Color::default(), npc.scale,
                );
            }
        }
        // (226 FlyingSnake rides the batch-4 bat arm
        // `49 | 51 | 150 | 152 | 226` above — d37's row, the shared
        // `:85797` arm; the 317/318/318/319 gore tail `:85830-85836` is
        // constant types, no argument draws.)
        // ---- batch 14 (the g8 row-gated activations, part 1: the desert
        // antlion + the d120 town pair + the bat pair + the creeper pair,
        // 7 kinds). g8 `82c93cec` (lane 1, the <=230 machine wiring) landed
        // 13 no-row kinds + the 64 PinkJellyfish artifact; d120 `c5e68a80`
        // landed the four town kinds; f42 `75f055fc` landed the Antlion.
        // `NPC.cs:86562-86565` — the Antlion (69): 50 type-250 sand puffs
        // at 2×dir/-2 (the shared 69/580/581/508/509/582 arm's num514=50
        // default; only 582 lowers it to 10 — g60's band). The 97/98 gore
        // pair `:86566-86570` is constant types, no argument draws. f42's
        // bestiary row (69) + g8's hand row — both activate the arm.
        69 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 250, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88163-88166` / `:88186-88189` — the Santa Claus (142) /
        // Steampunker (178): 50 type-5 puffs at 2.5×dir/-2.5 (the town
        // shape; d120's hand rows 142/178 — the landed truth, d36 skips
        // them per the keep-one note). The shimmer/altTexture gore gates
        // (`:88167-88174` 1338/157 + 1340/158×2 + 1341/159×2;
        // `:88190-88196` 1348/231 + 1349/232×2 + 1350/233) are field
        // reads + constants, no argument draws.
        142 | 178 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // (150 IceBat / 152 GiantFlyingFox ride the batch-4 bat arm
        // `49 | 51 | 150 | 152 | 226` above — g8's rows, the shared
        // `:85797` arm.)
        // `NPC.cs:87783-87786` — the Wall Creeper (164) / Wall Creeper
        // Wall (165): 50 type-18 puffs at 2.5×dir/-2.5 (the spider shape;
        // g8's hand rows). The 207/208×4 gore tail `:87787-87791` is
        // constant types, no argument draws.
        164 | 165 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 18, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 15 (the g8/d120 row-gated activations, part 2 — the
        // ACTIVATIONS-2 newly-rowed set COMPLETE at 17 kinds; 158 rides
        // the extended 82 wraith arm above, 179 the extended 83/84
        // cursed arm above — the same 6 gore-ARGUMENT draws).
        // `NPC.cs:88272-88275` / `:88438-88441` — the Truffle (160) /
        // Cyborg (209): 50 puffs at 2.5×dir/-2.5 (the town shape; d120's
        // hand rows). 160's dust is type 26 (the mushroom spore), 209's
        // type 5 — separate arms, the C# writes them separately. The
        // shimmer gore ladders (`:88276-88280` 1366/188 + 1367/189×2 +
        // 1368/190×2; `:88442-88446` 1294/289 + 1295/290×2 + 1296/291×2)
        // are field reads + constants, no argument draws.
        160 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 26, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        209 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:86419-86422` — the Herpling (174) / Floaty Gross (182):
        // 50 type-5 puffs at 2×dir/-2, `this.alpha` (the shared `:86409`
        // 173/174/181/182/268 arm; g8's rows; 173/181 stay row-gated —
        // d36's band, 268 g60's). The 225/226 + 238/239 gore tails
        // `:86428-86441` are constant types, no argument draws.
        174 | 182 => {
            for _ in 0..50 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0,
                    npc.alpha as i32, Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:85733-85736` — the Derpling (177): 15 type-5 puffs at
        // 2×dir/-2 (g8's row). The 234/235/236/236 gore tail
        // `:85737-85740` is constant types, no argument draws.
        177 => {
            for _ in 0..15 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // `NPC.cs:88830-88836` — the Ice Elemental (169): 100 type-92
        // frost puffs at (0,0), scale = 1.5 (the post-write OVERRIDES the
        // pool jitter), velocity×2 + noGravity (g8's row). Dust-only,
        // zero gore — the whole census.
        169 => {
            for _ in 0..100 {
                let idx = dust.new_dust(
                    rng, ctx, world, pos, w, h, 92, 0.0, 0.0, 0,
                    Color::default(), 1.5,
                );
                let d = dust.slot_mut(idx);
                d.scale = 1.5;
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `NPC.cs:86797-86800` — the Lost Girl (195) / Nymph (196): 75
        // type-5 puffs at 2×dir/-2 (g8's rows — the 195→196 Transform
        // keeps both kinds rowed). The 254/255×2/256×2 gore ladder
        // `:86801-86805` is constant types + `this.scale`, no argument
        // draws.
        195 | 196 => {
            for _ in 0..75 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 5, 2.0 * dir, -2.0, 0,
                    Color::default(), 1.0,
                );
            }
        }
        // ---- batch 13 (the d37 tail: the sea pair + the pirate, 3 kinds
        // — the lane's newly-rowed set COMPLETE at 23 activations).
        // `NPC.cs:88513-88516` — the Sea Snail (220) / Squid (221): 20
        // type-151 puffs at 2.5×dir/-2.5, alpha 0, scale 0.7 (d37's rows).
        // The 297 / 298/299 gore tail `:88517-88525` is constant types +
        // `this.scale`, no argument draws.
        220 | 221 => {
            for _ in 0..20 {
                dust.new_dust(
                    rng, ctx, world, pos, w, h, 151, 2.5 * dir, -2.5, 0,
                    Color::default(), 0.7,
                );
            }
        }
        // `NPC.cs:88129-88143` — the Pirate (229): the `IsShimmerVariant`
        // gate (`NPC.cs:6713-6723`: variation_index == 1 AND
        // `ShimmerTownTransform[229]`, true in the set) selects the
        // dust branch — the shimmer death draws 50 type-180 puffs at
        // the corpse's velocity, vel×2 + noGravity + scale = 1.4
        // (`:88131-88137`); the plain death draws 50 type-5 puffs at
        // 2.5×dir/-2.5 (`:88140-88143`, the town shape). The altTexture
        // gore gate `:88144-88151` (328/329/329/330/330) is a field read
        // + constants, no argument draws. d37's row.
        229 => {
            if npc.variation_index == 1
                && crate::npc_ai_town::shimmer_town_transform(npc_type)
            {
                for _ in 0..50 {
                    let idx = dust.new_dust(
                        rng, ctx, world, pos, w, h, 180, npc.vx, npc.vy, 0,
                        Color::default(), 1.0,
                    );
                    let d = dust.slot_mut(idx);
                    d.velocity *= 2.0;
                    d.no_gravity = true;
                    d.scale = 1.4;
                }
            } else {
                for _ in 0..50 {
                    dust.new_dust(
                        rng, ctx, world, pos, w, h, 5, 2.5 * dir, -2.5, 0,
                        Color::default(), 1.0,
                    );
                }
            }
        }
        _ => return false,
    }
    true
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests_union_b {
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    fn corpse(kind: NpcKind) -> Npc {
        let mut n = Npc::new(kind, 100.0, 100.0, 1, 0.0);
        n.hp = 0;
        n.last_hit_dir = -1;
        n.vx = 2.0;
        n.vy = -1.0;
        n
    }

    /// `NPC.cs:85065-85077` — the bees' death: 6 type-150 puffs, each the
    /// 6 default-block rolls then the Next(2) gate (noGravity+1.5 vs 0.8).
    /// The hand-rolled ladder pins the exact draw order on the stream.
    #[test]
    fn w4_g59_the_bee_death_draws_six_gated_puffs() {
        let w = air();
        let n = corpse(NpcKind::Bee);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        assert!(death_dust(&n, 210, &mut dust, &ctx(), &mut rng, &w));
        // The twin: the C# draw ladder by hand.
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let mut twin = UnifiedRandom::new(7);
        let mut gates = Vec::new();
        for _ in 0..6 {
            let _px = twin.next_max(bw.max(5) - 4);
            let _py = twin.next_max(bh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            gates.push(twin.next_max(2) == 0);
        }
        assert_eq!(rng, twin, "the bee death spends exactly 6×(6+1) draws");
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 6, "the flat 6 dust 150");
        for (d, &gate) in live.iter().zip(&gates) {
            assert_eq!(d.dust_type, 150);
            if gate {
                assert!(d.no_gravity);
                assert_eq!(d.scale, 1.5);
            } else {
                assert!(!d.no_gravity);
                assert_eq!(d.scale, 0.8);
            }
        }
    }

    /// `NPC.cs:89487-89501` — the Meteor Head: the 50-puff selector loop
    /// then the 50-ember loop, replayed on a twin pool (the type-6 tweak
    /// draw rides inside `new_dust`). 100 slots, the embers noGravity.
    #[test]
    fn w4_g59_the_meteor_head_death_draws_two_fifty_loops() {
        let w = air();
        let n = corpse(NpcKind::MeteorHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        let mut twin_rng = rng.clone();
        let mut twin = DustPool::new();
        assert!(death_dust(&n, 23, &mut dust, &ctx(), &mut rng, &w));
        let (mw, mh) = n.size();
        let (mw, mh) = (mw as i32, mh as i32);
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let mut ty = 25;
            if twin_rng.next_max(2) == 0 {
                ty = 6;
            }
            twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, mw, mh, ty, -2.0, -2.0, 0,
                Color::default(), 1.0,
            );
        }
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, mw, mh, 6, 0.4, -0.2, 100,
                Color::default(), 2.5,
            );
            let d = twin.slot_mut(idx);
            d.velocity *= 6.0;
            d.no_gravity = true;
        }
        assert_eq!(rng, twin_rng, "the two loops mirror the C# draws");
        assert_eq!(dust.slots(), twin.slots(), "slot-for-slot equality");
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 100);
        assert!(live[..50].iter().all(|d| d.dust_type == 25 || d.dust_type == 6));
        assert!(live[50..].iter().all(|d| d.dust_type == 6 && d.no_gravity));
    }

    /// `NPC.cs:85370-85375` — the snowman trio: all three ids draw the
    /// SAME 50×76 census (noGravity, scale×1.2, SpeedY -1).
    #[test]
    fn w4_g59_the_snowman_trio_shares_one_death_census() {
        let w = air();
        let n = corpse(NpcKind::SnowmanGangsta);
        let mut states = Vec::new();
        for ty in [143, 144, 145] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(11);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            for d in &live {
                assert_eq!(d.dust_type, 76);
                assert!(d.no_gravity);
            }
            // The ×1.2 write: the twin's untouched scale ×1.2.
            let mut twin = DustPool::new();
            let mut twin_rng = UnifiedRandom::new(11);
            let (sw, sh) = n.size();
            for _ in 0..50 {
                let idx = twin.new_dust(
                    &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                    sw as i32, sh as i32, 76, -1.0, -1.0, 0, Color::default(), 1.0,
                );
                let base = twin.slot(idx).scale;
                let d = twin.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.2;
                assert_eq!(d.scale, base * 1.2);
            }
            assert_eq!(rng, twin_rng);
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]), "one arm, three ids");
    }

    /// `NPC.cs:85350-85355` — the 147/184 pair: 30 × (76 + 4) pairs, the
    /// 4s tinted (97,200,255,100) alpha 100, both SpeedY -2.
    #[test]
    fn w4_g59_the_ice_slime_death_draws_thirty_pairs() {
        let w = air();
        let n = corpse(NpcKind::IceSlime);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        assert!(death_dust(&n, 147, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 60);
        for (i, d) in live.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(d.dust_type, 76);
                assert!(d.no_gravity);
            } else {
                assert_eq!(d.dust_type, 4);
                assert_eq!(d.alpha, 100);
                assert_eq!(d.color, Color::new(97, 200, 255, 100));
            }
        }
        // 184 draws the identical census.
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(5);
        assert!(death_dust(&n, 184, &mut dust2, &ctx(), &mut rng2, &w));
        assert_eq!(rng, rng2);
        assert_eq!(dust.slots(), dust2.slots());
    }

    /// `NPC.cs:85420-85424` — the Crimslime: 50 type-136 puffs at
    /// `this.alpha` (the ctor's 55 row) with velocity×2; the color
    /// stand-in is default (the module doc).
    #[test]
    fn w4_g59_the_crimslime_death_reads_the_alpha_row() {
        let w = air();
        let n = corpse(NpcKind::Crimslime);
        assert_eq!(n.alpha, 55, "the SetDefaults alpha row (`:11342`)");
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        assert!(death_dust(&n, 183, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 136 && d.alpha == 55));
        // The ×2 write against a twin's untouched velocity.
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(9);
        let (cw, ch) = n.size();
        for i in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                cw as i32, ch as i32, 136, -1.0, 0.0, 55, Color::default(), 1.0,
            );
            let base = twin.slot(idx).velocity;
            assert_eq!(live[i].velocity, base * 2.0);
        }
        assert_eq!(rng, twin_rng);
    }

    /// `NPC.cs:85490-85494` — the Illuminant Slime (138): 50 type-71
    /// puffs, alpha 200, velocity×1.5.
    #[test]
    fn w4_g59_the_illuminant_death_draws_fifty_seventy_ones() {
        let w = air();
        let n = corpse(NpcKind::IlluminantSlime);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        assert!(death_dust(&n, 138, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 71 && d.alpha == 200));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(13);
        let (iw, ih) = n.size();
        for i in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                iw as i32, ih as i32, 71, -1.0, 0.0, 200, Color::default(), 1.0,
            );
            let base = twin.slot(idx).velocity;
            assert_eq!(live[i].velocity, base * 1.5);
        }
        assert_eq!(rng, twin_rng);
    }

    /// The cull refusal (`Dust.cs:97-101`): an off-screen corpse's spawns
    /// return the sentinel; the argument draws (the bees' 6 gates) still
    /// run and the post-writes land on `Main.dust[6000]` — the C# shape.
    #[test]
    fn w4_g59_the_offscreen_cull_writes_the_sentinel_slot() {
        let w = air();
        let n = corpse(NpcKind::BeeSmall);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        let far = SpawnCtx {
            screen_position: Vec2::new(100000.0, 100000.0),
            ..ctx()
        };
        assert!(death_dust(&n, 211, &mut dust, &far, &mut rng, &w));
        let mut twin = UnifiedRandom::new(17);
        for _ in 0..6 {
            let _gate = twin.next_max(2);
        }
        assert_eq!(rng, twin, "only the 6 gate draws survive the cull");
        assert!(dust.slots()[..6000].iter().all(|d| !d.active));
        // The sentinel carries the LAST gate's write (the C# writes it too).
        let s = dust.slot(tr_core::particles::DUST_SENTINEL);
        assert!(s.scale == 1.5 || s.scale == 0.8);
    }

    /// The batch gate: a kind outside the match draws nothing and returns
    /// false (the gore-bearing arms are d55's; 231+ is g60's).
    #[test]
    fn w4_g59_a_non_batch_kind_draws_nothing() {
        let w = air();
        let n = corpse(NpcKind::Zombie);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        let before = rng.clone();
        assert!(!death_dust(&n, 3, &mut dust, &ctx(), &mut rng, &w));
        assert_eq!(rng, before, "no arm, no draws");
        assert!(dust.slots().iter().all(|d| !d.active));
    }

    /// `NPC.cs:86459-86462` — the Demon Eye: 50 type-5 puffs at
    /// 2×hitDirection/-2; the gore tail (1 + 2, `:86500-86501`) draws no
    /// arguments, so the twin replay is the dust loop alone.
    #[test]
    fn w4_g59_the_demon_eye_death_draws_fifty_type_fives() {
        let w = air();
        let n = corpse(NpcKind::DemonEye);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        assert!(death_dust(&n, 2, &mut dust, &ctx(), &mut rng, &w));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(23);
        let (dw, dh) = n.size();
        for _ in 0..50 {
            twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                dw as i32, dh as i32, 5, -2.0, -2.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(rng, twin_rng, "the gore tail draws no arguments");
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:87512-87515` / `:87566-87569` / `:87632-87635` — the
    /// type-18 trio (6/7/13): 50 puffs at `this.alpha`/`this.scale`,
    /// SpeedX = hitDirection (NOT doubled), SpeedY -2. All three ids draw
    /// the identical census on the same ctor state.
    #[test]
    fn w4_g59_the_type18_trio_reads_alpha_and_scale() {
        let w = air();
        let mut n = corpse(NpcKind::EaterOfSouls);
        n.alpha = 200;
        n.scale = 1.3;
        let mut states = Vec::new();
        for ty in [6, 7, 13] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(29);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            for d in &live {
                assert_eq!(d.dust_type, 18);
                assert_eq!(d.alpha, 200);
                // The pool's scale = (1 + Next(-20,21)*0.01) × this.scale.
                assert!((d.scale - 1.3).abs() <= 0.26 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]), "one shape, three ids");
    }

    /// `NPC.cs:87600-87603` — the Giant Worm head: 10 type-5 puffs at
    /// 2.5×dir/-2.5 (the 2.5 multiplier, NOT 2 — the worm family's own
    /// arm, `this.type - 7 + 18` gore draws nothing).
    #[test]
    fn w4_g59_the_giant_worm_death_draws_ten_at_two_point_five() {
        let w = air();
        let n = corpse(NpcKind::GiantWormHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        assert!(death_dust(&n, 10, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 5));
        // The SpeedX argument: -2.5 + the ±2 jitter (dir = -1).
        assert!(live.iter().all(|d| d.velocity.x >= -4.5 && d.velocity.x <= -0.5));
        assert!(live.iter().all(|d| d.velocity.y >= -4.5 && d.velocity.y <= -0.5));
    }

    /// `NPC.cs:87662-87665` etc. — the town quartet (17/18/19/20): 50
    /// type-5 puffs at 2.5×dir/-2.5 each; the shimmer/altTexture gore
    /// gates are field reads (no draws), so all four ids share the census.
    #[test]
    fn w4_g59_the_town_quartet_shares_one_census() {
        let w = air();
        let n = corpse(NpcKind::Merchant);
        let mut states = Vec::new();
        for ty in [17, 18, 19, 20] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:87351-87354` — the Servant of Cthulhu: 20 type-5 puffs
    /// (the 20-count arm, not 50).
    #[test]
    fn w4_g59_the_servant_death_draws_twenty() {
        let w = air();
        let n = corpse(NpcKind::ServantOfCthulhu);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        assert!(death_dust(&n, 5, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 20);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:88537-88540` — the Skeleton/Dark Caster: 20 type-26 bone
    /// puffs at 2.5×dir/-2.5; both ids share the big skeleton arm's dust
    /// head (the gore ladder is constant data).
    #[test]
    fn w4_g59_the_skeleton_pair_draws_twenty_bones() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut states = Vec::new();
        for ty in [21, 32] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(43);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20);
            assert!(live.iter().all(|d| d.dust_type == 26));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:89514-89519` — the Fire Imp: 50 type-6 embers at the
    /// corpse's velocity, scale 2.5, noGravity + velocity×2 (the type-6
    /// tweak draw rides inside the pool).
    #[test]
    fn w4_g59_the_fire_imp_death_draws_fifty_embers() {
        let w = air();
        let n = corpse(NpcKind::FireImp);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        let mut twin_rng = rng.clone();
        let mut twin = DustPool::new();
        assert!(death_dust(&n, 24, &mut dust, &ctx(), &mut rng, &w));
        let (fw, fh) = n.size();
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, Vec2::new(n.x, n.y),
                fw as i32, fh as i32, 6, n.vx, n.vy, 100, Color::default(), 2.5,
            );
            let d = twin.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 2.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 6 && d.no_gravity));
    }

    /// `NPC.cs:89560-89563` — the goblin quartet (26/27/28/29): 50 type-5
    /// puffs each; the per-kind gore-type ladder is constant data, so all
    /// four ids share the census.
    #[test]
    fn w4_g59_the_goblin_quartet_shares_one_census() {
        let w = air();
        let n = corpse(NpcKind::GoblinPeon);
        let mut states = Vec::new();
        for ty in [26, 27, 28, 29] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(53);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:89177-89180` — the Skeletron head/hand: 150 type-26 bone
    /// puffs at 2.5×dir/-2.5 (the largest batch-3 burst); the RedHat gore
    /// gate reads ai[3]/localAI[3], no argument draws.
    #[test]
    fn w4_g59_the_skeletron_pair_draws_one_fifty_bones() {
        let w = air();
        let n = corpse(NpcKind::SkeletronHead);
        let mut states = Vec::new();
        for ty in [35, 36] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(59);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 150);
            assert!(live.iter().all(|d| d.dust_type == 26));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:87932-87935` — the Guide: 50 type-5 puffs at 2.5×dir/-2.5.
    #[test]
    fn w4_g59_the_guide_death_draws_fifty() {
        let w = air();
        let n = corpse(NpcKind::Guide);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        assert!(death_dust(&n, 22, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:85807-85810` — the bat pair (49/51): 15 type-5 puffs at
    /// 2×dir/-2; both ids share the bat arm's dust head.
    #[test]
    fn w4_g59_the_bat_pair_draws_fifteen() {
        let w = air();
        let n = corpse(NpcKind::CaveBat);
        let mut states = Vec::new();
        for ty in [49, 51] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(67);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 15);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:86740-86743` — the Shark: 75 type-5 puffs at 2×dir/-2
    /// (the 75-count arm, the largest type-5 burst).
    #[test]
    fn w4_g59_the_shark_death_draws_seventy_five() {
        let w = air();
        let n = corpse(NpcKind::Shark);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        assert!(death_dust(&n, 65, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 75);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// `NPC.cs:89712-89715` — the Hornet: 50 type-18 puffs at
    /// `this.alpha`/`this.scale`, SpeedX = dir (the type-18 shape, batch
    /// 2's trio census on a new kind).
    #[test]
    fn w4_g59_the_hornet_death_draws_fifty_type_eighteens() {
        let w = air();
        let n = corpse(NpcKind::Hornet);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(73);
        assert!(death_dust(&n, 42, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 18));
    }

    /// `NPC.cs:86381-86384` / `:89803-89806` — the Piranha (10) and
    /// Voodoo Demon (50): the 2×dir/-2 type-5 shape at two counts.
    #[test]
    fn w4_g59_the_piranha_and_voodoo_counts() {
        let w = air();
        for (kind, ty, count, seed) in [
            (NpcKind::Piranha, 58, 10usize, 79),
            (NpcKind::VoodooDemon, 66, 50, 83),
        ] {
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    /// `NPC.cs:88921-88932` — the Wyvern head: 10 type-16 puffs (scale
    /// 1.5, vel×2, noGravity) then the gore tail's ARGUMENT draws — the
    /// Next(1,4) count + Next(11,14) per gore. The hand-rolled ladder
    /// pins the exact draw order (the gore bodies are d55's, NOT drawn).
    #[test]
    fn w4_g59_the_wyvern_death_consumes_the_gore_argument_draws() {
        let w = air();
        let n = corpse(NpcKind::WyvernHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(89);
        assert!(death_dust(&n, 87, &mut dust, &ctx(), &mut rng, &w));
        // The twin: the dust ladder + the gore argument draws by hand.
        let (ww, wh) = n.size();
        let (ww, wh) = (ww as i32, wh as i32);
        let mut twin = UnifiedRandom::new(89);
        for _ in 0..10 {
            let _px = twin.next_max(ww.max(5) - 4);
            let _py = twin.next_max(wh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let count = twin.next_range(1, 4);
        for _ in 0..count {
            let _ty = twin.next_range(11, 14);
        }
        assert_eq!(rng, twin, "the wyvern spends the dust + gore-arg ladder");
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        assert!(live.iter().all(|d| d.dust_type == 16 && d.no_gravity));
    }

    /// `NPC.cs:87446-87449` — the Leech head: 10 type-5 puffs at dir/-1
    /// (the SpeedY stays -1, the leech arm's own value — NOT the -2 the
    /// other 10-count arms use).
    #[test]
    fn w4_g59_the_leech_death_keeps_speed_y_minus_one() {
        let w = air();
        let n = corpse(NpcKind::LeechHead);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        assert!(death_dust(&n, 117, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 10);
        // SpeedY = -1 + the ±2 jitter → [-3, 1].
        assert!(live.iter().all(|d| d.velocity.y >= -3.0 && d.velocity.y <= 1.0));
        // SpeedX = dir(-1) + jitter → [-3, 1].
        assert!(live.iter().all(|d| d.velocity.x >= -3.0 && d.velocity.x <= 1.0));
    }

    /// `NPC.cs:87880-87883` / `:87912-87915` — the town families
    /// (106/107/108 and 123/124): 50 type-5 puffs at 2.5×dir/-2.5 each;
    /// all five ids share the census.
    #[test]
    fn w4_g59_the_town_families_share_one_census() {
        let w = air();
        let n = corpse(NpcKind::Wizard);
        let mut states = Vec::new();
        for ty in [106, 107, 108, 123, 124] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(101);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:86459-86462` — the eye pair (190/191): 50 type-5 puffs at
    /// 2×dir/-2 (the demon-eye arm's shape on the new kinds).
    #[test]
    fn w4_g59_the_eye_pair_draws_fifty() {
        let w = air();
        let n = corpse(NpcKind::CataractEye);
        let mut states = Vec::new();
        for ty in [190, 191] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(103);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:86459-86462` — the eye trio (192/193/194): the demon-eye
    /// arm's 50-count census on the remaining eye kinds.
    #[test]
    fn w4_g59_the_eye_trio_draws_fifty() {
        let w = air();
        let n = corpse(NpcKind::DialatedEye);
        let mut states = Vec::new();
        for ty in [192, 193, 194] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(107);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:88458-88461` — the pirate family (212-216): 50 type-5
    /// puffs each; the per-kind gore ladder is constant data, so all five
    /// ids share the census.
    #[test]
    fn w4_g59_the_pirate_family_shares_one_census() {
        let w = air();
        let n = corpse(NpcKind::PirateDeckhand);
        let mut states = Vec::new();
        for ty in [212, 213, 214, 215, 216] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(109);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:88378-88381` / `:88401-88404` — the Dye Trader / Party
    /// Girl: 50 type-5 puffs at 2.5×dir/-2.5 each.
    #[test]
    fn w4_g59_the_dye_and_party_girls_draw_fifty() {
        let w = air();
        for (kind, ty) in [(NpcKind::DyeTrader, 207), (NpcKind::PartyGirl, 208)] {
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(113);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    /// `NPC.cs:87491-87494` — the Queen Bee: 50 type-18 puffs at
    /// `this.alpha`/`this.scale`, SpeedX = dir (the type-18 shape).
    #[test]
    fn w4_g59_the_queen_bee_death_draws_fifty_type_eighteens() {
        let w = air();
        let n = corpse(NpcKind::QueenBee);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        assert!(death_dust(&n, 222, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 18));
    }

    /// `NPC.cs:85038-85041` — the Umbrella Slime: 50 type-4 puffs at
    /// 2×dir/-2, alpha 100, tinted (0,80,255,100).
    #[test]
    fn w4_g59_the_umbrella_slime_death_draws_tinted_fours() {
        let w = air();
        let n = corpse(NpcKind::UmbrellaSlime);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        assert!(death_dust(&n, 225, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        for d in &live {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.color, Color::new(0, 80, 255, 100));
        }
    }

    /// `NPC.cs:88086-88089` / `:88109-88112` — the Painter / Witch
    /// Doctor: 50 type-5 puffs at 2.5×dir/-2.5 each.
    #[test]
    fn w4_g59_the_painter_and_witch_doctor_draw_fifty() {
        let w = air();
        for (kind, ty) in [(NpcKind::Painter, 227), (NpcKind::WitchDoctor, 228)] {
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(137);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    // ---- batches 8-10 (the d35 row-gated activations). The d35 kinds
    // (GoblinScout 73 … PossessedArmor 140) land with the d35 merge; the
    // witnesses drive the RAW C# ids over a stand-in corpse shell (the
    // batch-1 snowman-trio precedent — `death_dust` matches on the type
    // argument, the shell only supplies the box/alpha/scale reads).

    /// `NPC.cs:88946-88951` — the mummy trio (78/79/80): 20 type-31
    /// puffs, scale 1.5, velocity×2 + noGravity; the wraith (82) shares
    /// the shape at type 54, alpha 50.
    #[test]
    fn w4_g59_the_mummy_trio_and_wraith_shapes() {
        let w = air();
        for (ty, alpha) in [(78, 0), (79, 0), (80, 0), (82, 50)] {
            let n = corpse(NpcKind::Skeleton);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(139);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20, "type {ty}");
            let want_ty = if ty == 82 { 54 } else { 31 };
            for d in &live {
                assert_eq!(d.dust_type, want_ty);
                assert_eq!(d.alpha, alpha);
                assert!(d.no_gravity);
                // scale = (1 + Next(-20,21)*0.01) × 1.5 ∈ [1.2, 1.8].
                assert!(d.scale >= 1.2 - 1e-6 && d.scale <= 1.8 + 1e-6);
                // The ×2 write: the pool's ±2 jitter doubled ∈ [-4, 4].
                assert!(d.velocity.x.abs() <= 4.0 + 1e-6);
                assert!(d.velocity.y.abs() <= 4.0 + 1e-6);
            }
        }
    }

    /// `NPC.cs:87152-87163` — the cursed pair (83/84): 20 type-31 puffs
    /// then the 3-gore tail's 6 `Next(-2,3)` ARGUMENT draws — the twin
    /// stream replays the exact ladder (the g63 gore-sibling rule).
    #[test]
    fn w4_g59_the_cursed_pair_consumes_the_gore_velocity_draws() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(149);
        assert!(death_dust(&n, 83, &mut dust, &ctx(), &mut rng, &w));
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let mut twin = UnifiedRandom::new(149);
        for _ in 0..20 {
            let _px = twin.next_max(bw.max(5) - 4);
            let _py = twin.next_max(bh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let gx = twin.next_range(-2, 3);
            let gy = twin.next_range(-2, 3);
            assert!((-2..3).contains(&gx) && (-2..3).contains(&gy));
        }
        assert_eq!(rng, twin, "20×6 dust draws + 6 gore argument draws");
        // 84 shares the census exactly.
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(149);
        assert!(death_dust(&n, 84, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng_b, twin);
        assert_eq!(dust.slots(), dust_b.slots());
    }

    /// `NPC.cs:85527-85531` — the Pixie (75): 50 type-55 puffs, alpha
    /// 200, velocity×2 (dust-only; the twin pool pins the ×2 write).
    #[test]
    fn w4_g59_the_pixie_death_draws_fifty_type_fifty_fives() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(151);
        assert!(death_dust(&n, 75, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(151);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                55, -1.0, 0.0, 200, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 2.0;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert!(live.iter().all(|d| d.dust_type == 55 && d.alpha == 200));
    }

    /// `NPC.cs:89560-89563` / `:88537-88540` — the goblin scout (73)
    /// 50×5 and the skeleton pair (77/110) 20×26 ride the batch-3 shapes.
    #[test]
    fn w4_g59_the_goblin_scout_and_skeleton_pair_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, want_ty, count) in [(73, 5, 50), (77, 26, 20), (110, 26, 20)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == want_ty));
        }
    }

    /// `NPC.cs:88768-88788` — the Mimic (85): the ai[3] selector picks
    /// the dust type (2→10, 3→37, else 7); 20 puffs, default args.
    #[test]
    fn w4_g59_the_mimic_death_reads_the_ai3_selector() {
        let w = air();
        for (ai3, want_ty) in [(0.0f32, 7), (2.0, 10), (3.0, 37), (1.0, 7)] {
            let mut n = corpse(NpcKind::Skeleton);
            n.ai[3] = ai3;
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(157);
            assert!(death_dust(&n, 85, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20);
            assert!(live.iter().all(|d| d.dust_type == want_ty), "ai3={ai3}");
        }
    }

    /// `NPC.cs:87616-87619` / `:87583-87586` / `:87512-87515` — the
    /// digger trio (95/96/97) 10×5 at 2.5×dir, the seeker trio
    /// (98/99/100) + Corruptor (94) 50×18 at dir/-2 reading the live
    /// alpha/scale; the unicorn (86) 50×5 and the giant bat (93) 15×5
    /// ride the type-5 shapes.
    #[test]
    fn w4_g59_the_digger_seeker_and_corruptor_arms() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 90;
        n.scale = 1.1;
        for ty in [95, 96, 97] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 10);
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
        for ty in [94, 98, 99, 100] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50);
            for d in &live {
                assert_eq!(d.dust_type, 18);
                assert_eq!(d.alpha, 90);
                assert!((d.scale - 1.1).abs() <= 0.22 + 1e-6);
            }
        }
        for (ty, count) in [(86, 50), (93, 15)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(163);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }


    /// `NPC.cs:87547-87550` — the Clinger (101): 50 type-18 puffs at
    /// dir/-2 reading the live alpha/scale (the EoSouls arm).
    #[test]
    fn w4_g59_the_clinger_death_reads_alpha_and_scale() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 90;
        n.scale = 1.1;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(173);
        assert!(death_dust(&n, 101, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        for d in &live {
            assert_eq!(d.dust_type, 18);
            assert_eq!(d.alpha, 90);
            assert!((d.scale - 1.1).abs() <= 0.22 + 1e-6);
        }
    }

    /// `NPC.cs:85509-85513` — the Gastropod (122): 50 type-72 puffs,
    /// alpha 200, velocity×1.5 (dust-only; the twin pool pins the ×1.5).
    #[test]
    fn w4_g59_the_gastropod_death_draws_fifty_type_seventy_twos() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(179);
        assert!(death_dust(&n, 122, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(179);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                72, -1.0, 0.0, 200, Color::default(), 1.0,
            );
            twin.slot_mut(idx).velocity *= 1.5;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        assert!(live.iter().all(|d| d.dust_type == 72 && d.alpha == 200));
    }

    /// `NPC.cs:89085-89090` — the Possessed Armor (140): 20 type-54
    /// puffs, alpha 50, scale 1.5, velocity×2 + noGravity (the death-only
    /// `life <= 0` gate, the wraith's shape).
    #[test]
    fn w4_g59_the_possessed_armor_death_draws_twenty_type_fifty_fours() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(181);
        assert!(death_dust(&n, 140, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 20);
        for d in &live {
            assert_eq!(d.dust_type, 54);
            assert_eq!(d.alpha, 50);
            assert!(d.no_gravity);
            assert!(d.scale >= 1.2 - 1e-6 && d.scale <= 1.8 + 1e-6);
        }
    }

    /// `NPC.cs:85554-85557` — the Green Jellyfish (103): 25 type-4 puffs
    /// at 2×dir/-2, alpha 0, tinted (70,225,140,100) — the shared arm's
    /// 103 color row (63/64 stay row-gated).
    #[test]
    fn w4_g59_the_green_jellyfish_death_draws_tinted_fours() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(167);
        assert!(death_dust(&n, 103, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 25);
        for d in &live {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 0);
            assert_eq!(d.color, Color::new(70, 225, 140, 100));
        }
    }

    // ---- batch 11 (the d34 row-gated activations). The d34 kinds
    // (AngryBones 31 … BlueJellyfish 63) land with the d34 merge; the
    // witnesses drive the RAW C# ids over a stand-in corpse shell (the
    // batches-8-10 precedent — `death_dust` matches on the type argument,
    // the shell only supplies the box/alpha/scale reads).

    /// `NPC.cs:88537-88540` — Angry Bones (31) rides the big skeleton
    /// arm's 20×26 census exactly (the d34 hand row; its gore path falls
    /// to the generic constant tail — zero argument draws).
    #[test]
    fn w4_g59_the_angry_bones_death_rides_the_skeleton_census() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(191);
        assert!(death_dust(&n, 31, &mut dust, &ctx(), &mut rng, &w));
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(191);
        assert!(death_dust(&n, 21, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng, rng_b, "31 and 21 spend the same draws");
        assert_eq!(dust.slots(), dust_b.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 20);
        assert!(live.iter().all(|d| d.dust_type == 26));
    }

    /// `NPC.cs:89132-89141` — the Cursed Skull (34): 15 × (15 + 26) pairs
    /// at -velocity×0.2, the 15s noGravity/noLight alpha 100 scale 1.8,
    /// the 26s noLight scale 0.9, both velocity×1.3 — the twin pool pins
    /// the exact writes (dust-only, zero gore, zero argument draws).
    #[test]
    fn w4_g59_the_cursed_skull_death_draws_fifteen_twin_pairs() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(193);
        assert!(death_dust(&n, 34, &mut dust, &ctx(), &mut rng, &w));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(193);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..15 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                15, -0.4, 0.2, 100, Color::default(), 1.8,
            );
            let d = twin.slot_mut(idx);
            d.no_light = true;
            d.no_gravity = true;
            d.velocity *= 1.3;
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                26, -0.4, 0.2, 0, Color::default(), 0.9,
            );
            let d = twin.slot_mut(idx);
            d.no_light = true;
            d.velocity *= 1.3;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 30);
        for pair in live.chunks(2) {
            assert_eq!(pair[0].dust_type, 15);
            assert!(pair[0].no_gravity && pair[0].no_light);
            assert_eq!(pair[0].alpha, 100);
            assert_eq!(pair[1].dust_type, 26);
            assert!(pair[1].no_light && !pair[1].no_gravity);
        }
    }

    /// `NPC.cs:88232-88235` / `:89109-89112` — the Old Man (37) 50×5 at
    /// 2.5×dir/-2.5 and the Bone Serpent head (39) 20×26 at 2.5×dir/-2.5;
    /// the 40/41 Hide-fold segments reach the feed as the head's cs_id 39
    /// (d34's fold) — the same burst, verified by driving 39 thrice.
    #[test]
    fn w4_g59_the_old_man_and_bone_serpent_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, want_ty, count) in [(37, 5, 50), (39, 26, 20)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(197);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == want_ty));
        }
    }

    /// `NPC.cs:89752-89755` — the Man Eater (43) / Snatcher (56): 50
    /// type-40 puffs at dir/-2 reading the live alpha, scale 1.2 (the
    /// pool's ±0.2 jitter over 1.2); both ids draw the same census.
    #[test]
    fn w4_g59_the_man_eater_pair_draws_fifty_type_forties() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 70;
        let mut states = Vec::new();
        for ty in [43, 56] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(199);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            for d in &live {
                assert_eq!(d.dust_type, 40);
                assert_eq!(d.alpha, 70);
                assert!(d.scale >= 1.0 - 1e-6 && d.scale <= 1.4 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert_eq!(states[0], states[1], "43 and 56 share the census");
    }

    /// `NPC.cs:86381-86384` / `:89786-89789` — the Corrupt Bunny (47)
    /// 10×5 at 2×dir/-2 (the piranha arm) and the Harpy (48) 50×5 (its
    /// own arm); both gore tails are constant types.
    #[test]
    fn w4_g59_the_corrupt_bunny_and_harpy_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, count) in [(47, 10), (48, 50)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(211);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    // ---- batch 12 (the d34 tail + c4/d23/d37 row-gated activations).
    // The witnesses drive the RAW C# ids over a stand-in corpse shell
    // (the batches-8-11 precedent).

    /// `NPC.cs:86381-86384` / `:86617-86620` / `:89803-89806` — the
    /// Corrupt Goldfish (57) 10×5 (the piranha arm), the Vulture (61)
    /// 50×5 and the Demon (62) 50×5 (the voodoo arm); all gore tails
    /// constant types.
    #[test]
    fn w4_g59_the_corrupt_goldfish_vulture_and_demon_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, count) in [(57, 10), (61, 50), (62, 50)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(223);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), count, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
        }
    }

    /// `NPC.cs:85554-85557` — the Blue Jellyfish (63): 25 type-4 puffs
    /// at 2×dir/-2, alpha 0, tinted (50,120,255,100) — the shared arm's
    /// DEFAULT color row (64 stays row-gated; 103's tint wired batch 10).
    #[test]
    fn w4_g59_the_blue_jellyfish_death_draws_the_default_tint() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(227);
        assert!(death_dust(&n, 63, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 25);
        for d in &live {
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.alpha, 0);
            assert_eq!(d.color, Color::new(50, 120, 255, 100));
        }
    }

    /// `NPC.cs:86514-86517` / `:89769-89772` — the Arapaima (157) 50×5
    /// at 2×dir/-2 and the Angry Trapper (175) 50×40 at dir/-2 reading
    /// the live alpha, scale 1.2 (the man-eater shape, its own arm).
    #[test]
    fn w4_g59_the_arapaima_and_angry_trapper_counts() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 70;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(229);
        assert!(death_dust(&n, 157, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(229);
        assert!(death_dust(&n, 175, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        for d in &live {
            assert_eq!(d.dust_type, 40);
            assert_eq!(d.alpha, 70);
            assert!(d.scale >= 1.0 - 1e-6 && d.scale <= 1.4 + 1e-6);
        }
    }

    /// `NPC.cs:87462-87465` / `:85807-85810` — the beetle trio
    /// (217/218/219) 50×18 at dir/-2 reading the live alpha/scale, and
    /// the Flying Snake (226) 15×5 (the bat arm); all gore tails
    /// constant types + `this.scale`.
    #[test]
    fn w4_g59_the_beetle_trio_and_flying_snake_counts() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 90;
        n.scale = 1.1;
        let mut states = Vec::new();
        for ty in [217, 218, 219] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(233);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            for d in &live {
                assert_eq!(d.dust_type, 18);
                assert_eq!(d.alpha, 90);
                assert!((d.scale - 1.1).abs() <= 0.22 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(233);
        assert!(death_dust(&n, 226, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 15);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    // ---- batch 13 (the d37 tail: the sea pair + the pirate).

    /// `NPC.cs:88513-88516` — the Sea Snail (220) / Squid (221): 20
    /// type-151 puffs at 2.5×dir/-2.5, scale 0.7 (the pool's ±0.2 jitter
    /// over 0.7); both ids share the census.
    #[test]
    fn w4_g59_the_sea_snail_and_squid_share_one_census() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut states = Vec::new();
        for ty in [220, 221] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(239);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 20, "type {ty}");
            for d in &live {
                assert_eq!(d.dust_type, 151);
                assert_eq!(d.alpha, 0);
                assert!(d.scale >= 0.5 - 1e-6 && d.scale <= 0.9 + 1e-6);
            }
            states.push(rng.peek());
        }
        assert_eq!(states[0], states[1]);
    }

    /// `NPC.cs:88129-88143` — the Pirate (229): the plain death draws
    /// 50×5 at 2.5×dir/-2.5; the shimmer death (variation_index == 1,
    /// the ShimmerTownTransform[229] set member) draws 50×180 at the
    /// corpse's velocity, vel×2 + noGravity + scale = 1.4 — the twin
    /// pool pins the exact writes.
    #[test]
    fn w4_g59_the_pirate_death_splits_on_the_shimmer_gate() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(241);
        assert!(death_dust(&n, 229, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
        n.variation_index = 1;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(241);
        assert!(death_dust(&n, 229, &mut dust, &ctx(), &mut rng, &w));
        let mut twin = DustPool::new();
        let mut twin_rng = UnifiedRandom::new(241);
        let (bw, bh) = n.size();
        let pos = Vec2::new(n.x, n.y);
        for _ in 0..50 {
            let idx = twin.new_dust(
                &mut twin_rng, &ctx(), &w, pos, bw as i32, bh as i32,
                180, 2.0, -1.0, 0, Color::default(), 1.0,
            );
            let d = twin.slot_mut(idx);
            d.velocity *= 2.0;
            d.no_gravity = true;
            d.scale = 1.4;
        }
        assert_eq!(rng, twin_rng);
        assert_eq!(dust.slots(), twin.slots());
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 180 && d.no_gravity && d.scale == 1.4));
    }

    // ---- batch 14 (the g8/d120/f42 row-gated activations, part 1).

    /// `NPC.cs:86562-86565` — the Antlion (69): 50 type-250 sand puffs
    /// at 2×dir/-2 (the shared arm's num514=50 default; the 97/98 gore
    /// pair is constant types — the dust loop is the whole census).
    #[test]
    fn w4_g59_the_antlion_death_draws_fifty_sand_puffs() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(251);
        assert!(death_dust(&n, 69, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 250 && d.alpha == 0));
        // The twin: the dust ladder by hand (zero argument draws).
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let mut twin = UnifiedRandom::new(251);
        for _ in 0..50 {
            let _px = twin.next_max(bw.max(5) - 4);
            let _py = twin.next_max(bh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        assert_eq!(rng, twin, "the antlion death spends exactly 50×6 draws");
    }

    /// `NPC.cs:88163-88166` / `:88186-88189` — the Santa Claus (142) /
    /// Steampunker (178): 50 type-5 puffs at 2.5×dir/-2.5 (the town
    /// shape, d120's rows); both share the census exactly (the shimmer
    /// gore gates are field reads).
    #[test]
    fn w4_g59_the_santa_and_steampunker_share_the_town_census() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut states = Vec::new();
        for ty in [142, 178] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(257);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert_eq!(states[0], states[1], "142 and 178 share the census");
    }

    /// `NPC.cs:85807-85810` — the Ice Bat (150) / Giant Flying Fox (152)
    /// ride the bat arm: 15 type-5 puffs, census-equal to 49/51/226.
    #[test]
    fn w4_g59_the_ice_bat_and_flying_fox_ride_the_bat_arm() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut states = Vec::new();
        for ty in [49, 150, 152, 226] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(263);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 15, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert!(states.windows(2).all(|p| p[0] == p[1]));
    }

    /// `NPC.cs:87783-87786` — the Wall Creeper pair (164/165): 50
    /// type-18 puffs at 2.5×dir/-2.5 (the spider shape; the 207/208×4
    /// gore tail is constant types — zero argument draws).
    #[test]
    fn w4_g59_the_wall_creeper_pair_draws_fifty_spider_puffs() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(269);
        assert!(death_dust(&n, 164, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 18 && d.alpha == 0));
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(269);
        assert!(death_dust(&n, 165, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng_b, rng);
        assert_eq!(dust.slots(), dust_b.slots());
    }

    // ---- batch 15 (the g8/d120 row-gated activations, part 2).

    /// `NPC.cs:88272-88275` / `:88438-88441` — the Truffle (160) 50×26
    /// (the mushroom spore) / Cyborg (209) 50×5, both at 2.5×dir/-2.5
    /// (the town shape, d120's rows); the shimmer gore ladders are
    /// field reads — zero argument draws.
    #[test]
    fn w4_g59_the_truffle_and_cyborg_town_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        for (ty, dust_ty) in [(160, 26), (209, 5)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(271);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == dust_ty && d.alpha == 0));
        }
    }

    /// `NPC.cs:86419-86422` — the Herpling (174) / Floaty Gross (182):
    /// 50 type-5 puffs reading the LIVE `this.alpha` (the shared
    /// `:86409` arm); both ids share the census exactly.
    #[test]
    fn w4_g59_the_herpling_and_floaty_gross_read_the_live_alpha() {
        let w = air();
        let mut n = corpse(NpcKind::Skeleton);
        n.alpha = 60;
        let mut states = Vec::new();
        for ty in [174, 182] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(277);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 50, "type {ty}");
            for d in &live {
                assert_eq!(d.dust_type, 5);
                assert_eq!(d.alpha, 60);
            }
            states.push(rng.peek());
        }
        assert_eq!(states[0], states[1], "174 and 182 share the census");
    }

    /// `NPC.cs:85733-85736` / `:88830-88836` — the Derpling (177) 15×5
    /// and the Ice Elemental (169) 100×92 (scale = 1.5 post-write
    /// OVERRIDES the jitter, vel×2 + noGravity; dust-only).
    #[test]
    fn w4_g59_the_derpling_and_ice_elemental_counts() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(281);
        assert!(death_dust(&n, 177, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 15);
        assert!(live.iter().all(|d| d.dust_type == 5));
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(281);
        assert!(death_dust(&n, 169, &mut dust, &ctx(), &mut rng, &w));
        let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 100);
        for d in &live {
            assert_eq!(d.dust_type, 92);
            assert_eq!(d.scale, 1.5, "the post-write override, not a jitter");
            assert!(d.no_gravity);
            assert!(d.velocity.x.abs() <= 4.0 + 1e-6);
            assert!(d.velocity.y.abs() <= 4.0 + 1e-6);
        }
    }

    /// `NPC.cs:86797-86800` / `:88971-88976` / `:87152-87163` — the
    /// Lost Girl (195) / Nymph (196) 75×5 pair, the Vampire Bat (158)
    /// riding the 82 wraith census, and the Crimson Axe (179) riding
    /// the 83 cursed census INCLUDING its 6 gore-ARGUMENT draws.
    #[test]
    fn w4_g59_the_nymph_pair_and_the_vampire_crimson_shared_arms() {
        let w = air();
        let n = corpse(NpcKind::Skeleton);
        let mut states = Vec::new();
        for ty in [195, 196] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(283);
            assert!(death_dust(&n, ty, &mut dust, &ctx(), &mut rng, &w));
            let live: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(live.len(), 75, "type {ty}");
            assert!(live.iter().all(|d| d.dust_type == 5));
            states.push(rng.peek());
        }
        assert_eq!(states[0], states[1]);
        // 158 ≡ 82 (20×54, alpha 50, scale 1.5 jittered, vel×2 noGrav).
        let mut dust_a = DustPool::new();
        let mut rng_a = UnifiedRandom::new(283);
        assert!(death_dust(&n, 82, &mut dust_a, &ctx(), &mut rng_a, &w));
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(283);
        assert!(death_dust(&n, 158, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng_a, rng_b);
        assert_eq!(dust_a.slots(), dust_b.slots());
        // 179 ≡ 83 including the 6 Next(-2,3) gore argument draws.
        let mut dust_a = DustPool::new();
        let mut rng_a = UnifiedRandom::new(283);
        assert!(death_dust(&n, 83, &mut dust_a, &ctx(), &mut rng_a, &w));
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(283);
        assert!(death_dust(&n, 179, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng_a, rng_b);
        assert_eq!(dust_a.slots(), dust_b.slots());
        let (bw, bh) = n.size();
        let (bw, bh) = (bw as i32, bh as i32);
        let mut twin = UnifiedRandom::new(283);
        for _ in 0..20 {
            let _px = twin.next_max(bw.max(5) - 4);
            let _py = twin.next_max(bh.max(5) - 4);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let _gx = twin.next_range(-2, 3);
            let _gy = twin.next_range(-2, 3);
        }
        assert_eq!(rng_b, twin, "20×6 dust draws + 6 gore argument draws");
    }

    // ---- batch 16 (the 15-base-fold head-wiring check).

    /// `NPC.cs:87870-87891` — the Bound Goblin (105) rides the
    /// 106/107/108 arm: 50 type-5 puffs at 2.5×dir/-2.5; the raw id
    /// 105 and the folded feed id 107 (the port's `entry_for_npc`
    /// fold) share the census exactly.
    #[test]
    fn w4_g59_the_bound_goblin_rides_the_tinkerer_arm() {
        let w = air();
        let n = corpse(NpcKind::BoundGoblin);
        assert_eq!(n.kind.cs_id(), 107, "the base's 105→107 fold");
        let mut dust_a = DustPool::new();
        let mut rng_a = UnifiedRandom::new(293);
        assert!(death_dust(&n, 105, &mut dust_a, &ctx(), &mut rng_a, &w));
        let mut dust_b = DustPool::new();
        let mut rng_b = UnifiedRandom::new(293);
        assert!(death_dust(&n, 107, &mut dust_b, &ctx(), &mut rng_b, &w));
        assert_eq!(rng_a, rng_b);
        assert_eq!(dust_a.slots(), dust_b.slots());
        let live: Vec<_> = dust_a.slots().iter().filter(|d| d.active).collect();
        assert_eq!(live.len(), 50);
        assert!(live.iter().all(|d| d.dust_type == 5));
    }

    /// The base-fold head-wiring census: every folded segment's cs_id
    /// resolves to its WIRED head — the feed fires the head's arm.
    /// 8/9→7, 11/12→10, 14/15→13, 88-92→87, 118/119→117 (the
    /// `entry_for_npc` folds; the 134 head stays UNWIRED — 139's
    /// cs_id folds to 134 and `death_dust(134)` returns false, f5's
    /// keep-one).
    #[test]
    fn w4_g59_the_worm_folds_resolve_to_wired_heads() {
        let w = air();
        for (kind, head) in [
            (NpcKind::DevourerBody, 7),
            (NpcKind::DevourerTail, 7),
            (NpcKind::GiantWormBody, 10),
            (NpcKind::GiantWormTail, 10),
            (NpcKind::EaterOfWorldsBody, 13),
            (NpcKind::EaterOfWorldsTail, 13),
            (NpcKind::WyvernLegs, 87),
            (NpcKind::WyvernTail, 87),
            (NpcKind::LeechBody, 117),
            (NpcKind::LeechTail, 117),
        ] {
            assert_eq!(kind.cs_id(), head, "{kind:?} folds onto {head}");
            let n = corpse(kind);
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(307);
            assert!(
                death_dust(&n, kind.cs_id() as i32, &mut dust, &ctx(), &mut rng, &w),
                "{kind:?}'s head arm fires"
            );
        }
        // The Destroyer head: NOT wired in this lane (f5's keep-one).
        let n = corpse(NpcKind::Probe);
        assert_eq!(n.kind.cs_id(), 134, "the 139→134 fold");
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(307);
        assert!(!death_dust(&n, 134, &mut dust, &ctx(), &mut rng, &w));
        assert_eq!(rng, UnifiedRandom::new(307), "the unwired head draws nothing");
    }
}
