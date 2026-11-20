# 65 — NpcKind coverage census, NPCID 1–350 (W4-f17, verify pass 1)

Date: 2026-11-20. Branch `task/W4-f17-npc-kinds-verify-1`, base `origin/main`
`a725acb0`. Doc-only census; no code touched.

## Method

- Rust roster: the 270 live `NpcKind::X => id` arms of `to_u16`
  (`crates/tr-app/src/npc/data.rs:219-589`, `#[deny(unreachable_patterns)]`
  bijection; the enum itself lives in `npc.rs`).
- Vanilla-id resolution per kind, in the port's own precedence:
  1. the `cs_id` hand rows (`data.rs:2992-3099`, 57 rows), then
  2. the bestiary `net_id` (`bestiary.rs` `ENTRIES`, 249 kind rows; the
     slime-variant constants resolve via `loot_db.rs:10-17`), then
  3. the folded families: worm/golem/wyvern/leech/destroyer/fishron-hazard
     segments fold their bestiary credit onto the head (`bestiary.rs:2257-2304`)
     but each segment kind still exists and owns its vanilla `NPCID` slot
     (e.g. `WyvernLegs` 88 … `WyvernTail` 92, `texture_net_id`,
     `render/entities.rs:1089-1094`; Destroyer 135/136/Probe 139,
     `entities.rs:1102-1105`). Those slots count as covered.
- `NpcKind::BoundGoblin` (`npc.rs:736-739`, cites `NPCID.BoundGoblin` 105)
  covers 105 even though its `cs_id` folds onto the Tinkerer's 107
  (`bestiary.rs:2352`); same pattern as BoundWizard 106 / BoundMechanic 123,
  which have direct hand rows.
- C# side: the `public const short` list of `decompiled/Terraria/ID/NPCID.cs`
  (`:10677-11377`, ids 0–350). Name-only differences do not affect coverage:
  the snowman trio's port names are shifted (`SnowBalla`→144 MisterStabby,
  `MysterySnowman`→145 SnowBalla, pinned by
  `the_frost_trio_credit_ids_follow_the_vanilla_npc_names`,
  `bestiary.rs:2426-2443`) and the pirate band uses port names
  (CaptainsMate/Archer/Corsair on vanilla 213/214/215,
  `bestiary.rs:1993-2029`) — all five ids 212–216 are covered.

## Result

- Kinds in `to_u16`: **270**; every kind resolves to a vanilla id or a folded
  family slot.
- Vanilla ids 1–350 with a NpcKind row: **165**.
- Vanilla ids 1–350 with **NO NpcKind row: 185** (list below). Two of them are
  vanilla placeholders, not real NPCs: 76 `None2`, 146 `None3` — effective
  real-NPC gap: **183**.
- Id 0 (`None`) is outside the census range; the port's `Slime` covers
  `BlueSlime` 1 and the negative-net-id colour family.

## Missing list by sibling-branch band (claimed vs unclaimed)

Claim check (`git branch -a`, `git worktree list`, `git log`): the five
`task/W4-d34..d38-npc-kinds-*` branches exist **locally only**, each parked at
`origin/main` `a725acb0` with **0 unique commits**; none is pushed
(`git log origin/task/W4-d34* …` → unknown revision). So every id below is
**UNCLAIMED on origin** — the local branch names reserve the bands, but no
pushed work covers any of them. (d35's worktree holds untracked census scratch
under `tools/_d35_*` — analysis in flight, nothing committed.) The one pushed
kinds branch, `origin/task/W4-d43-npc-kinds-631-697`, is also 0 commits ahead
of main and its range is outside 1–350.

### d34 band, ids 1–70 — 20 missing (all unclaimed)

31 AngryBones, 34 CursedSkull, 37 OldMan, 39 BoneSerpentHead, 40 BoneSerpentBody,
41 BoneSerpentTail, 43 ManEater, 46 Bunny, 47 CorruptBunny, 48 Harpy, 56 Snatcher,
57 CorruptGoldfish, 61 Vulture, 62 Demon, 63 BlueJellyfish, 64 PinkJellyfish,
67 Crab, 68 DungeonGuardian, 69 Antlion, 70 SpikeBall

### d35 band, ids 71–140 — 34 missing (all unclaimed)

73 GoblinScout, 74 Bird, 75 Pixie, **76 None2 (placeholder)**, 77 ArmoredSkeleton,
78 Mummy, 79 DarkMummy, 80 LightMummy, 82 Wraith, 83 CursedHammer,
84 EnchantedSword, 85 Mimic, 86 Unicorn, 93 GiantBat, 94 Corruptor, 95 DiggerHead,
96 DiggerBody, 97 DiggerTail, 98 SeekerHead, 99 SeekerBody, 100 SeekerTail,
101 Clinger, 102 AnglerFish, 103 GreenJellyfish, 104 Werewolf, 109 Clown,
110 SkeletonArcher, 112 VileSpit, 116 TheHungryII, 120 ChaosElemental, 121 Slimer,
122 Gastropod, 137 IlluminantBat, 140 PossessedArmor

### d36 band, ids 141–210 — 50 missing (all unclaimed)

141 ToxicSludge, 142 SantaClaus, **146 None3 (placeholder)**, 148 Penguin,
149 PenguinBlack, 150 IceBat, 151 Lavabat, 152 GiantFlyingFox, 153 GiantTortoise,
154 IceTortoise, 155 Wolf, 156 RedDevil, 157 Arapaima, 158 VampireBat, 159 Vampire,
160 Truffle, 162 Frankenstein, 163 BlackRecluse, 164 WallCreeper, 165 WallCreeperWall,
166 SwampThing, 167 UndeadViking, 168 CorruptPenguin, 169 IceElemental,
170 PigronCorruption, 171 PigronHallow, 172 RuneWizard, 173 Crimera, 174 Herpling,
175 AngryTrapper, 176 MossHornet, 177 Derpling, 178 Steampunker, 179 CrimsonAxe,
180 PigronCrimson, 181 FaceMonster, 182 FloatyGross, 185 SnowFlinx, 195 LostGirl,
196 Nymph, 197 ArmoredViking, 198 Lihzahrd, 199 LihzahrdCrawler, 201 HeadacheSkeleton,
202 MisassembledSkeleton, 203 PantlessSkeleton, 204 SpikedJungleSlime, 205 Moth,
206 IcyMerman, 209 Cyborg

### d37 band, ids 211–280 — 46 missing (all unclaimed)

217 CochinealBeetle, 218 CyanBeetle, 219 LacBeetle, 220 SeaSnail, 221 Squid,
224 FlyingFish, 226 FlyingSnake, 229 Pirate, 230 GoldfishWalker, 231 HornetFatty,
232 HornetHoney, 233 HornetLeafy, 234 HornetSpikey, 235 HornetStingy,
236 JungleCreeper, 237 JungleCreeperWall, 238 BlackRecluseWall, 239 BloodCrawler,
240 BloodCrawlerWall, 241 BloodFeeder, 242 BloodJelly, 243 IceGolem, 244 RainbowSlime,
251 Eyezor, 252 Parrot, 253 Reaper, 254 ZombieMushroom, 255 ZombieMushroomHat,
256 FungoFish, 257 AnomuraFungus, 258 MushiLadybug, 259 FungiBulb, 260 GiantFungiBulb,
268 IchorSticker, 269 RustyArmoredBonesAxe, 270 RustyArmoredBonesFlail,
271 RustyArmoredBonesSword, 272 RustyArmoredBonesSwordNoArmor, 273 BlueArmoredBones,
274 BlueArmoredBonesMace, 275 BlueArmoredBonesNoPants, 276 BlueArmoredBonesSword,
277 HellArmoredBones, 278 HellArmoredBonesSpikeShield, 279 HellArmoredBonesMace,
280 HellArmoredBonesSword

### d38 band, ids 281–350 — 35 missing (all unclaimed)

281 RaggedCaster, 282 RaggedCasterOpenCoat, 283 Necromancer, 284 NecromancerArmored,
285 DiabolistRed, 286 DiabolistWhite, 287 BoneLee, 289 GiantCursedSkull, 290 Paladin,
291 SkeletonSniper, 292 TacticalSkeleton, 293 SkeletonCommando, 294 AngryBonesBig,
295 AngryBonesBigMuscle, 296 AngryBonesBigHelmet, 297 BirdBlue, 298 BirdRed,
299 Squirrel, 300 Mouse, 301 Raven, 302 SlimeMasked, 303 BunnySlimed, 304 HoppinJack,
316 Ghost, 317 DemonEyeOwl, 318 DemonEyeSpaceship, 322 SkeletonTopHat,
323 SkeletonAstonaut, 324 SkeletonAlien, 328 PumpkingBlade, 333 SlimeRibbonWhite,
334 SlimeRibbonYellow, 335 SlimeRibbonGreen, 336 SlimeRibbonRed, 337 BunnyXmas

## Notes for the kind-wave leads

- The covered 165 include the port's above-350 families' in-range members only
  where a kind exists (e.g. the DD2/lunar/martian families live at 381+ and do
  not fill any 1–350 hole).
- Folded segments are covered kinds but carry `cs_id` = the head's id; any
  statue/`MechSpawn` work on them must read the kind, not `cs_id`
  (`data.rs:3030-3037` gate note).
- `TheHungryII` 116 is a distinct vanilla id from `TheHungry` 115 (the port's
  `Hungry` covers 115 only) — a genuine one-id gap inside an otherwise ported
  boss family.
