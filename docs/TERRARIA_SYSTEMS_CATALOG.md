# Terraria Systems Catalog — Complete Porting Reference

**Purpose:** Detailed mapping of all Terraria C# systems to Rust implementation targets  
**Reference:** Decompiled sources `Terraria.sln` (1,549 .cs files from Terraria + 167 from ReLogic)  
**Last Updated:** 2026-10-29

---

## 📊 System Inventory Summary

| Category | Files | Classes/Methods | Lines of Code | Priority |
|----------|-------|-----------------|---------------|----------|
| **World & Tiles** | ~120 | ~850 | ~45,000 | P0 - Core |
| **NPCs & Enemies** | ~65 | ~780 AI behaviors | ~85,000 | P1 - Core |
| **Player** | ~30 | ~1,200 methods | ~55,000 | P1 - Core |
| **Items & Crafting** | ~90 | ~50,000 lines logic | ~65,000 | P2 - Essential |
| **Projectiles** | ~20 | ~1,100 types | ~40,000 | P2 - Important |
| **World Gen** | ~150 | ~200 generators | ~90,000 | P2 - Major |
| **Liquids** | ~8 | ~150 methods | ~10,000 | P1 - Core |
| **Lighting** | ~5 | ~100 methods | ~15,000 | P0 - Core |
| **Background/Sky** | ~15 | ~250 draw calls | ~12,000 | P1 - Visual |
| **UI & Menu** | ~250 | ~3,000 controls | ~120,000 | P3 - Nice |
| **Audio** | ~10 | ~500 sounds | ~8,000 | P3 - Nice |
| **Networking** | ~80 | Multiplayer logic | ~50,000 | P2 - Later |

---

## 🌍 World & Tile Systems (P0 - CORE)

### `World.cs` & `WorldGen.cs`
**Location:** `Terraria/`, `Terraria.GameContent/Generation/`  
**Scope:** World generation, tile placement, world save/load

#### Completed
- ✅ Basic terrain generation (heightmap, surface, dirt, stone)
- ✅ Cave generation (noise-based)
- ✅ Ore placement (coal, copper, iron, gold, diamond)
- ✅ Forest lake generation
- ✅ Lava puddles (Hell layer)
- ✅ Snow biome surface generation
- ✅ Desert biome generation
- ✅ Tree spawning (normal, snow, pine)
- `.trwld` save format parser/writer

#### In Progress / TODO
- ❌ **Hardmode spread mechanics** (`WorldGen.Hardmod`)
- ❌ **Corruption/Crimson biomes** (`WorldGen.CorruptOrCrims`)
- ❌ **Jungle biome** (`WorldGen.Jungle`)
- ❌ **Hallow biome** (`WorldGen.Hallow`)
- ❌ **Dungeon generation** (`WorldGen.Dungeon`)
- ❌ **Jungle temple** (`WorldGen.Temples`)
- ❌ **Living trees** (`WorldGen.LivingTrees`)
- ❌ **Pyramid generation** (`WorldGen.Pyramids`)
- ❌ **Ocean structures** (anchors, sandstone)
- ❌ **Chest/generator loot** (`WorldGen.ItemDrop`)
- ❌ **Waterfalls/Half-block spouts**
- ❌ **Tile update events** (liquid flow triggers, redstone logic)

### `Liquid.cs` (4 files)
**Location:** `Terraria/Liquid.*`  
**Scope:** Water, lava, honey flow simulation

#### Completed
- ✅ Liquid cell simulation (gravity + 6-neighbor leveling)
- ✅ Flow rules (30 Hz tick rate)
- ✅ Kill/retire overflow handling
- ✅ Surface liquid detection
- ✅ Basic renderer (visible levels, edge walls, waterfall fade)
- ✅ Light color interpolation from liquids
- ✅ Forest lake and lava puddle seeding

#### TODO
- ❌ **Waterfall_*** overlay rendering
- ❌ **Shimmer sparkle effects**
- ❌ **Water style frames** (different materials)
- ❌ **Liquid acceleration physics** (full NavMesh simulation)
- ❌ **Actuator liquid control**
- ❌ **Buckets** (empty/full mechanics)

### `Main.cs` (Tile Drawing)
**Location:** `Terraria/Main.cs`  
**Scope:** Tile autotiling, framing, drawing order

#### Completed
- ✅ Generic 20-frame block autotiling
- ✅ Merge families (Dirt/Mud/Ash/Grass/Ore blocks)
- ✅ 2,228 merge pairs auto-extracted
- ✅ Wall framing (20×3 lookup grid)
- ✅ Tree composition (22 px trunks, branches)
- ✅ Half-brick sloping
- ✅ Grass overhang framing
- ✅ Tile light scanner with seeds

#### TODO
- ❌ **TileFrameImportant multi-tile objects** (doors, chests, furniture)
- ❌ **Per-type special branches** (moss, vines, cactus arms)
- ❌ **Glowstone/gemspark light emission**
- ❌ **Paint tint blending**
- ❌ **Background wall autotiling**
- ❌ **Platform framing** (wood, glass, metal variants)
- ❌ **Banner placement rules**

---

## 👾 NPC & Enemy Systems (P1 - CORE)

### `NPC.cs` (~780 NPC types)
**Location:** `Terraria/NPC.cs`  
**Scope:** All enemy, boss, and town NPC behaviors

#### Completed
- ✅ Basic hopper AI (slimes)
- ✅ Flight patterns (demon eyes)
- ✅ Pathfinding/walking (zombies)
- ✅ Day/night spawn rules
- ✅ Biome spawn checks
- ✅ Contact damage system
- ✅ Knockback/resistance
- ✅ Boss spawning (King Slime)
- ✅ Minion summoning
- ✅ Overhead HP bars
- ✅ Coin drop/purse system

#### TODO — By Category

##### **Bosses (25+)**
- ❌ Queen Slime
- ❌ Eater of Worlds/Brain of Cthulhu
- ❌ Skeletron
- ❌ Deerclops
- ❌ Queen Bee
- ❌ SGT. Shimmyface
- ❌ Plantera
- ❌ Golem
- ❌ Empress of Light
- ❌ Moon Lord
- ❌ All intermediate bosses

##### **Minibosses (20+)**
- ❌ Duke Fishron
- ❌ Paintra
- ❌ Martian Saucer
- ❌ Cultists
- ❌ Lunatic Cultist

##### **Common Enemies (700+)**
- **Slime family:** Blue, red, green, purple, black, yellow, rainbow, ice, hell slimes
- **Bat family:** Red bats, ice bats, hell bats, voodoo bats
- **Worm/Vine:** Snakes, serpents, viper, worm sections
- **Flying:** Bats, fairies, jellyfish, wraiths, harpies
- **Ground:** Zombies, skeletons, dune worms, sand sharks
- **Aerial:** Banshees, wyverns, phoenixes, pixies
- **Undead:** Ghosts, spirits, vampires, demons
- **Elemental:** Fire elementals, ice golems, rock elements
- **Jungle:** Piranhas, spitters, snakes, bees
- **Cavern:** Rock giants, cave fish, floating eyes
- **Underground:** Crystals, shadows, voids
- **Event-specific:** Christmas, halloween, solar eclipse, frost legion

##### **Town NPCs (50+)**
- ❌ Merchant, nurse, arms dealer, demolitionist
- ❌ Goblin tinkerer, witch doctor, painter
- ❌ Dryad, party girl, maid, tax collector
- ❌ Steampunker, wizard, merchant of darkness
- ❌ Clothier, mechanic, gnome, santa claus
- ❌ Truffle, cyborg, alien, ladybug
- ❌ **Shop inventory systems** (all NPC vendors)
- ❌ **AI behavior** (idle, follow player, avoid danger)
- ❌ **Happiness/move-in mechanics**
- ❌ **Gift giving/dialogue**

##### **AI Behavior Trees**
Each NPC type has complex state machines:
- ❌ Idle wandering
- ❌ Patrol routes
- ❌ Chase/flee distances
- ❌ Attack cooldowns
- ❌ Projectile spawning
- ❌ Teleport/blink abilities
- ❌ Phase transitions (bosses)
- ❌ Spawn condition logic
- ❌ Death animations/drops

---

## 🧑 Player System (P1 - CORE)

### `Player.cs` (~55k lines)
**Location:** `Terraria/Player.cs`  
**Scope:** Player physics, animation, survival, equipment

#### Completed
- ✅ Gravity/acceleration/friction
- ✅ Jump mechanics (single + hold-to-jump)
- ✅ 1-tile auto-step
- ✅ AABB tile collision resolution
- ✅ Water buoyancy/drowning
- ✅ Fall/lava damage
- ✅ Health regeneration (out of combat)
- ✅ Camera follow algorithm
- ✅ Starter character composition
- ✅ Item use swing arc
- ✅ Mining/placement/reach checks
- ✅ Classic HUD (hotbar, hearts, boss bar, breath)
- ✅ Respawn mechanics
- ✅ Hurt cooldown/knockback

#### TODO

##### **Player Rendering Layers** (`PlayerDrawLayers`)
- ❌ Full armor rendering (helmet, chestplate, legs)
- ❌ Accessory display (wings, accessories)
- ❌ Composite arm rotation (walking, jumping, mining swings)
- ❌ Hair/back-clip rendering
- ❌ Mount/glider wings animation
- ❌ Head gear/items in hand
- ❌ Vanity slot rendering

##### **Player Status Effects**
- ❌ Buff system (20+ active buffs)
- ❌ Debuff system (poisoned, cursed, burning, frozen, etc.)
- ❌ Potion sickness timers
- ❌ Cooldown tracking
- ❌ Stat bonuses calculation

##### **Mounts & Gliders**
- ❌ Mount entry/exit mechanics
- ❌ Mount movement quirks (hover, dash, speed boost)
- ❌ Glider deployment
- ❌ Wings/flapping animation
- ❌ Jetpack fuel mechanics

##### **Advanced Movement**
- ❌ Gravity flip states
- ❌ Spider climbing walls
- ❌ Hook/grappling physics
- ❌ Rocket boots/jellyfish boots
- ❌ Flippers swimming
- ❌ Ice skating
- ❌ Sand sinking
- ❌ Water elevator mechanics

##### **Item Animation States**
- ❌ Full `itemAnimation` enum (70+ states)
- ❌ Use timings per item type
- ❌ Auto-swing mechanics
- ❌ Right-click alternative uses

---

## 🎒 Items & Crafting (P2 - IMPORTANT)

### `Item.cs` & `Recipe.cs`
**Location:** `Terraria/Item.cs`, `Terraria.GameContent/Recipes/`  
**Scope:** All 550+ items, crafting recipes, inventory management

#### Completed
- ✅ 10-slot dynamic hotbar
- ✅ LMB mining with per-tool timers
- ✅ RMB placement with reach/checks
- ✅ Item drops as physics entities
- ✅ Pickup magnet into slots
- ✅ Tool tiers (basic wood → endgame)
- ✅ 5×5 re-framing placement preview
- ✅ Local storage `.trplayer` sidecar

#### TODO

##### **Tools & Weapons** (~800 items)
- ❌ **Tool tiers and durability** (pickaxes, axes, hammers, swords, spears)
- ❌ **Special tool powers** (night-vision, auto-mine, area mine)
- ❌ **Magic weapons** (mana cost, fire rates, projectiles)
- ❌ **Ranged weapons** (ammo consumption, velocity, spread)
- ❌ **Summon weapons** ( minion count, AI)
- ❌ **Boomerangs** (return arc, hits)
- ❌ **Yoyos** (spin pattern, damage ticks)
- ❌ **Flails** (chain length, hit zones)
- ❌ **Thrown explosives** (grenades, dynamite)
- ❌ **Bows/crossbows** (arrow types, charge)
- ❌ **Throwables** (bottles, shurikens, knives)

##### **Blocks & Furniture** (~800 items)
- ❌ **Ore blocks** (colored, smelted variants)
- ❌ **Wood types** (normal, emerald, sapphire, topaz, ruby, amber, diamond, palm)
- ❌ **Bricks** (clay, stone, mud, desert, jungle)
- ❌ **Plates** (frost, marble, granite, obsidian)
- ❌ **Glass variants** (tinted, stained)
- ❌ **Platform variants** (wood, iron, glass, pressure plates)
- ❌ **Furniture sets** (tables, chairs, beds, shelves)
- ❌ **Decorations** (lanterns, rugs, curtains, plants)
- ❌ **Walls** (indoor/outdoor, dungeon, jungal)
- ❌ **Signs and billboards**

##### **Consumables** (~400 items)
- ❌ **Potions** (health, mana, buffs, crafting)
- ❌ **Food** (regeneration, buffs)
- ❌ **Materials** (ores, bars, gems, leather, bone)
- ❌ **Hooks** (summon food, bait)
- ❌ **Spawn eggs** (NPCs, critters)
- ❌ **Key items** (boss summons, progression keys)
- ❌ **Seeds and planting materials**
- ❌ **Dyes and coloring**

##### **Crafting System**
- ❌ **Full recipe registry** (2,500+ recipes)
- ❌ **Crafting menu UI** (category tabs, search)
- ❌ **Auto-craft** functionality
- ❌ **Recipe unlocking** (unlock on first craft)
- ❌ **Prefix system** (random roll modifiers)
- ❌ **Reforge stations** (weapons, armor)
- ❌ **Alchemy table**, **anvil**, **melting pot**
- ❌ **Workbench context requirements**

##### **Equipment Slots**
- ❌ **Head/Helm** (20+ slots)
- ❌ **Body/Chest** (armor pieces)
- ❌ **Legs/Pants** (armored pants, belts)
- ❌ **Accessories** (15 slots, modifier rolls)
- ❌ **Neck/Ear/Waist** vanity slots
- ❌ **Trinkets** (special accessory category)
- ❌ **Stat calculation** (attack, defense, magic, speed)

##### **Inventory UI**
- ❌ **Main inventory** (50+ slots, expandable)
- ❌ **Accessory slots UI**
- ❌ **Equipment panel** (equipment + vanity toggle)
- ❌ **Chests** (40+ slots, shared bank)
- ❌ **Fish tanks**, **grave markers**
- ❌ **Sorting/filtering**
- ❌ **Stack splitting/combining**

---

## 💥 Projectiles System (P2 - IMPORTANT)

### `Projectile.cs` (~1,100 types)
**Location:** `Terraria/Projectile.cs`  
**Scope:** All thrown/spawned projectiles and their AI

#### Completed
- ✅ Shuriken bounce physics
- ✅ Basic gravity (fireballs, arrows)
- ✅ Damage contact vs piercing
- ✅ Knockback application
- ✅ Hitbox timing/deactivation
- ✅ Particle spawns on hit

#### TODO — Projectile Families

##### **Arrows/Bolts** (~50 types)
- ❌ Normal, explosive, corrosive, freezing, holy, night
- ❌ Homing mechanisms
- ❌ Piercing (multi-hit arrows)
- ❌ Return mechanics (boomerang arrows)

##### **Spells** (~200 types)
- ❌ Magic missiles (flame, frost, lightning)
- ❌ Summoned minions (wandering, attacking, hovering)
- ❌ Staff summons (snowball, tentacles, vines)
- ❌ Area-of-effect spells (explosives, blasts)

##### **Melee Throwables** (~100 types)
- ❌ Yoyo spin patterns
- ❌ Flail chain swings
- ❌ Boomerang return arcs
- ❌ Spear throws
- ❌ Throwing knives/shurikens
- ❌ Grenade explosions (varied sizes)

##### **Ranged Projectiles** (~300 types)
- ❌ Arrows, bullets, bolts (different velocities)
- ❌ Shotgun spread patterns
- ❌ Machine gun multi-shot
- ❌ Mine launchers, rocket launchers
- ❌ Sniper rifle penetration

##### **Enemy Dropped** (~400 types)
- ❌ Slime tears
- ❌ Bat bites
- ❌ Skeleton skulls
- ❌ Demon eye fireballs
- ❌ Boss-specific (laser beams, tears, tentacle strikes)
- ❌ Event projectiles (skeleton rain, icicles)

##### **AI Behaviors**
Each projectile has unique AI:
- ❌ Gravity/free-fall
- ❌ Homing guidance
- ❌ Spin/rotation
- ❌ Lifetime expiration
- ❌ Piercing vs single-target
- ❌ Bounce counts (walls, tiles)
- ❌ Splash damage radius
- ❌ Debuff application (on hit)
- ❌ Minion targeting (select enemies)

---

## 🎨 Background & Sky Systems (P1 - VISUAL)

### `Main.cs` (Background) & `Background.cs`
**Location:** `Terraria/GameContent/Background/`  
**Scope:** Parallax backgrounds, sky layers, weather

#### Completed
- ✅ Background gradient colors
- ✅ Deterministic cloud generation
- ✅ Sun, moon, stars sprite rendering
- ✅ Day-night clock (15 min day, 9 min night)
- ✅ Per-column cave fade
- ✅ Rain system (fall direction, splashes, particles)
- ✅ Forest background (7/8 mountains, tree lines)
- ✅ Snow biome (pines, snow surface)
- ✅ Ocean horizon (sea band)
- ✅ Cloud banks and storm clouds
- ✅ Lightning flashes

#### TODO

##### **Background Variants**
- ❌ **Corruption purple/black themes** (`BG_12..15`)
- ❌ **Crimson blood/red themes** (`BG_16..19`)
- ❌ **Jungle green/gray themes** (`BG_20..23`)
- ❌ **Hallow pastel/blue themes** (`BG_24..27`)
- ❌ **Underground caverns** (rock, crystal caves)
- ❌ **Desert dunes and canyons**
- ❌ **Dungeon blue/gold theme**
- ❌ **Ice biome teal palette**
- ❌ **Mushroom biome glow**
- ❌ **Hell/fire underworld**

##### **Sky Elements**
- ❌ **Sun/moon position calculations**
- ❌ **Star cluster patterns**
- ❌ **Rainbow generation**
- ❌ **Eclipse sky darkening**
- ❌ **Blood moon sky** (red gradient)
- ❌ **Solar/nebula/eclipse events**
- ❌ **Frost meteor shower**
- ❌ **Rainbow clouds** (after rain event)

##### **Weather Effects**
- ❌ **Snow biome precipitation**
- ❌ **Acid rain** (underground corruption)
- ❌ **Wind gusts** (cloud movement speed)
- ❌ **Thunder audio sync**
- ❌ **Blizzard intensity variations**

##### **Dynamic Events**
- ❌ **Seasonal decorations** (halloween, christmas)
- ❌ **Anniversary fireworks**
- ❌ **Celestial events** (starfall, comet)
- ❌ **Boss intro sequences**
- ❌ **Daybreak/sunset animations**

---

## 🔦 Lighting System (P1 - CORE)

### `TileLightScanner.cs` & `LightMap.cs`
**Location:** `Terraria/Lighting/`  
**Scope:** Per-tile lighting, light map blur, global brightness

#### Completed
- ✅ `TileLightScanner` seed values (surface/hell/wall/emissive/liquid)
- ✅ `LightMap` two-pass four-sweep blur
- ✅ Per-mask decay factors
- ✅ `Main.SetBackColor` → `ApplyColorOfTheSkiesToTiles`
- ✅ Per-column surface seeding
- ✅ Global Brightness = 1.2
- ✅ `GetColor9Slice`/`GetColor4Slice` block slicing
- ✅ Four LightModes (Color/White/Retro/Trippy)
- ✅ Mouse-light flicker (dynamic source)
- ✅ F2 mode toggling

#### TODO

##### **Environmental Light Sources**
- ❌ **Sunlight intensity** (time-based fading)
- ❌ **Moonlight phases** (full/half/new moon)
- ❌ **Star light contribution**
- ❌ **Torches/torched light spread**
- ❌ **Lamps/chandeliers**
- ❌ **Fire place/cribbage**
- ❌ **Campfires** (buff aura)

##### **Tile-Specific Light**
- ❌ **Glowstone/gemspark emission levels**
- ❌ **Magma/liquid light flicker**
- ❌ **Crystal/ore internal glow**
- ❌ **Lantern light direction**
- ❌ **Candle flame animation**

##### **Global Effects**
- ❌ **Lightning flash bursts**
- ❌ **Explosion temporary brighteners**
- ❌ **Portal opening light**
- ❌ **Shadow light suppression**
- ❌ **God ray volumetric light**

---

## 🪵 Tile Objects & Special Mechanics (P1-P2)

### `TileObjectData.cs` & Related Systems
**Location:** `Terraria/TileObjectType.cs`, `Terraria.GameContent/TileObjects/`  
**Scope:** Multi-tile placement, alignment, connectivity

#### TODO

##### **Furniture Tile Objects**
- ❌ **Doors** (open/close animation, frame anchoring)
- ❌ **Chests** (two-block vertical stack)
- ❌ **Tables** (tabletop + legs positioning)
- ❌ **Chairs** (sit-on mechanics)
- ❌ **Beds** (sleep time skip, head/foot)
- ❌ **Lamps/Shelves** (wall mounting)
- ❌ **Anvils/Workbenches** (crafting station)
- ❌ **Cooking pots/cauldrons**
- ❌ **Looms**, **kilns**, **furnaces**

##### **Connectivity**
- ❌ **Platform connections** (continuous horizontal surfaces)
- ❌ **Sign placement rules** (must attach to solid)
- ❌ **Wall-mounted fixtures** (must have backing wall)
- ❌ **Door framing** (clearance above doorframe)

##### **Paint & Tinting**
- ❌ **Block paint coverage** (per-block tint)
- ❌ **Paint bucket consumption**
- ❌ **Paint fade on break**
- ❌ **Coating system** (shimmer coating on water)

---

## 🎵 Audio System (P3 - NICE)

### `SoundEffectReader.cs` & `XACT`
**Location:** `Terraria/GameContent/Audio/`  
**Scope:** Sound effects, music playback

#### Completed
- ✅ PCM sound effect loading
- ✅ Eight original cues through `rodio`
- ✅ Explosions, jumps, mining, UI clicks
- ✅ Basic spatial 2D positioning

#### TODO

##### **Music System**
- ❌ **XACT/WMA decode** (original audio format)
- ❌ **Music selector** (biome/time/event switching)
- ❌ **Crossfade transitions**
- ❌ **Boss music trigger**
- ❌ **Event-specific tracks** (eclipse, invasion)
- ❌ **Underground variant mixing**

##### **3D Audio**
- ❌ **Distance attenuation** (volume drop-off)
- ❌ **Directional panning** (left/right balance)
- ❌ **Occlusion** (blocked by solid blocks)
- ❌ **Liquid audio modifiers** (splash underwater)

---

## 🖥️ UI & Menu Systems (P3 - LATER)

### `Main.cs`, `GUI.cs`, `Interface/*.cs`
**Location:** `Terraria/UI/` (~250 files)  
**Scope:** All GUI panels, menus, overlays

#### TODO — Major Systems
- ❌ **Pause menu** (settings, exit, save prompt)
- ❌ **Character creation/editing** (name, appearance)
- ❌ **Settings panels** (video, audio, controls)
- ❌ **Social screen** (friends, chat, Discord)
- ❌ **Achievement popups**
- ❌ **Quest journal** (active objectives)
- ❌ **Bestiary entries** (enemy compendium)
- ❌ **Achievement unlocked dialogs**

---

## 🌐 Networking (P2 - LATER MULTIPLAYER)

### `NetHandler.cs` & Multiplayer Logic
**Location:** `Terraria/Net/`, `Terraria.GameContent/Multiplayer/`  
**Scope:** TCP/IP multiplayer, synchronization

#### TODO
- ❌ **Host-client architecture**
- ❌ **Entity state interpolation**
- ❌ **Block break/place synchronization**
- ❌ **NPC/player damage packets**
- ❌ **Item pickup broadcasts**
- ❌ **Chat message relay**
- ❌ **Exact 1.4.5.8 protocol matching**

---

## 📚 Implementation Notes

### Decompilation Source Locations
```bash
# Primary Terraria logic
decompiled/Terraria.dll

# Game content (world gen, recipes, assets)  
decompiled/Terraria.GameContent.dll

# Item/Tile/NPC ID mappings
decompiled/Terraria.ID.dll
```

### Naming Convention for Rust Modules
Match original C# class names:
- `NPC.cs` → `crates/tr-npc/src/npc.rs`
- `Liquid.cs` → `crates/tr-core/src/liquid.rs`
- `TileObjectData` → `crates/tr-world/src/tile_objects.rs`

### Priority Guidelines
- **P0**: Rendering core (tiles, lighting, liquids) - must match pixel-perfect
- **P1**: Survival gameplay (player, NPCs, basic combat) - critical for sandbox feel
- **P2**: Content depth (items, crafting, more NPCs) - makes game fun
- **P3**: Polish (UI, audio, networking) - enhances experience but optional early

---

**This catalog is auto-updated as tasks are completed.**  
**Agents: Check here before starting work to find appropriate scope.**
