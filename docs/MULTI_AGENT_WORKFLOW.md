# Multi-Agent Workflow — Instructions for AI Agents

**Purpose:** Protocol for coordinated parallel development of Terraria-Rust  
**Read First:** All agents must read `agents.md` and this file before starting work

---

## 🎯 Your Role as Agent

You are one of multiple AI agents working in parallel on different aspects of the Terraria port. Each agent may be assigned specific tasks simultaneously, but **conflicts must be avoided**.

---

## 📋 Workflow Protocol (STRICT)

### Step 1: Before Starting Any Work
1. **Read these files:**
   - `agents.md` - overall repository rules
   - `docs/TASK_STATUS.md` - current task assignments
   - `docs/TERRARIA_SYSTEMS_CATALOG.md` - available work items
   - `docs/ROADMAP.md` - milestone context

2. **Check TASK_STATUS.md:**
   - Find all unassigned tasks ("❌ Pending" + "-" in Assigned To column)
   - Identify which systems you could potentially work on
   - Verify no other agent has claimed similar work

3. **Choose ONE task to focus on:**
   - Prefer tasks matching your expertise/strengths
   - Consider what's blocking downstream work
   - Don't grab too many tasks simultaneously

### Step 2: Claiming a Task
1. **Update TASK_STATUS.md:**
   ```markdown
   Change status from "❌ Pending" → "🔄 In Progress"
   Add your identifier in "Assigned To" column
   Example: | M4-NPC-5 | Zombie AI behavior | 🔄 In Progress | Agent-XYZ | Comment here |
   ```

2. **Note:** 
   - Be specific about your name/ID so others know who owns it
   - If working autonomously, use something like "Agent-Auto"
   - Update timestamp in header

3. **Create a Git branch** (if committing changes):
   ```bash
   git checkout -b feature/task-id-description
   # Example: git checkout -b feature/M4-NPC-5-zombie-ai
   ```

### Step 3: Implementation
1. **Follow the Golden Rule** (`agents.md`):
   - Locate original C# implementation in `decompiled/`
   - Read entire function and understand edge cases
   - Port faithfully first (same structure, same math, same order)
   - Name Rust functions after C# originals
   - Reference mapping in one-line comment

2. **Document unusual decisions:**
   - `docs/ANALYSIS.md` if there were non-obvious findings
   - `docs/TASK_STATUS.md` Notes column for progress updates

3. **Verify before finishing:**
   - `cargo test --workspace` passes
   - Visual work: compare screenshots with original
   - Performance work: measure with `TR_BENCH`

### Step 4: Completing a Task
1. **Mark as done in TASK_STATUS.md:**
   ```markdown
   Status: "✅ Done"
   Notes: "Committed in <hash> or <PR URL>"
   Timestamp: update header
   ```

2. **Commit changes:**
   ```bash
   git add .
   git commit -m "[M4-NPC-5] Implement zombie AI behavior
    
    - Ported from Terraria/NPC.cs AI_003_Fighters
    - Added day-burn mechanics verified against source
    - Matches decompiled constants and RNG order"
   ```

3. **Push to remote:**
   ```bash
   git push origin feature/task-id-description
   ```

4. **Update NOTES in TASK_STATUS.md:**
   - Link to commit/PR
   - Mention any known issues or TODOs left over

5. **Free yourself from the task:**
   ```markdown
   Assigned To: "- (completed)"
   ```

### Step 5: Choose Next Task
- Return to Step 1
- Pick another pending task
- Repeat cycle

---

## 🚫 DO NOT DO THIS

### Conflicts to Avoid
```
❌ Two agents working on same NPC type
❌ Duplicate implementation of liquid flow physics
❌ Multiple agents touching same file simultaneously
❌ Overlapping world generation passes
❌ Same projectile family implemented twice
```

### Common Mistakes
```
❌ Guessing behavior without checking C# source
❌ Optimizing before verifying correctness
❌ Using different naming conventions than original
❌ Missing edge cases from original code
❌ Not documenting why decisions were made
```

---

## 📂 File Organization Reference

### Where Things Go
```
crates/tr-core/src/         # Core engine systems
  tiledata.rs              # Tile types, frames, merges
  lighting.rs              # Light map, scanner, blur
  liquid.rs                # Water/lava simulation
  
crates/tr-world/src/        # World management
  generator.rs             # Terrain generation
  chunk.rs                 # Chunk storage
  tile_objects.rs          # Furniture, doors, chests

crates/tr-npc/src/          # NPC logic
  ai.rs                    # All NPC behaviors
  spawn_rules.rs           # Biome/time/spawn logic
  
crates/tr-player/src/       # Player system
  physics.rs               # Movement, collision
  draw_layers.rs           # Armor, accessories, composite
  
crates/tr-items/src/        # Item logic
  item_data.rs             # Registry, stats
  crafting.rs              # Recipe system
  inventory.rs             # UI, slots
  
crates/tr-projectiles/src/  # Projectile system
  ai.rs                    # Projectile behaviors
  damage.rs                # Contact, piercing
```

### Naming Conventions
```rust
// Module names match C# class names
pub mod npc;           // Terraria/NPC.cs
pub mod player;        // Terraria/Player.cs  
pub mod liquid;        // Terraria/Liquid.cs
pub mod projectile;    // Terraria/Projectile.cs

// Functions reference original
fn ai_003_fighters() { /* Terraria/NPC.AI_003_Fighters */ }
fn update_liquid() { /* Terraria/Liquid.UpdateLiquid */ }

// Comments map back to source
// Source: Terraria.GameContent.Events.BossEvent
```

---

## 🔧 Tools & Commands

### Verification Commands
```bash
# Run all tests (required before commit)
cargo test --workspace

# Benchmark frame timing
TR_BENCH=600 cargo run

# Compare screenshots pixel-perfect
python tools/pixel_diff.py expected.png actual.png

# Force screenshot at load point
TR_SHOT=1 cargo run

# Debug lighting calculation
TR_LIGHT_AT=1 cargo run

# Measure light cache performance
TR_PAN=0.25 TR_BENCH=600 cargo run
```

### Finding C# Originals
```bash
# Search decompiled sources for implementation
cd decompiled/
grep -r "AI_003_Fighters" Terraria/NPC.cs
grep -r "GetColor9Slice" Terraria/Main.cs

# List all NPC AI methods
grep "^public void AI_" Terraria/NPC.cs | head -50

# Find recipe definitions
find Terraria/GameContent/Recipes -name "*.cs" -exec grep -l "KingSlime" {} \;
```

---

## 💡 Decision Framework

### When Uncertain About Behavior
1. **Search C# source FIRST** - don't guess
2. Look at related callers of the function
3. Check constants/tables referenced
4. Note special cases and edge conditions
5. Document uncertainties in `docs/ANALYSIS.md`

### When Tasks Conflict
1. Stop immediately
2. Check `TASK_STATUS.md` timestamps
3. Determine who started first (timestamp comparison)
4. The earlier agent keeps their assignment
5. Second agent picks different task
6. Resolve via human intervention only if needed

### Scope Boundaries
- **One agent per system category per week:** NPCs, Items, Projectiles, etc.
- **Don't touch adjacent files** unless explicitly part of your task
- **Coordinate through commits** - read recent PR history before modifying shared areas

---

## 📈 Tracking Progress

### For Yourself
- Maintain personal task log in comments
- Track time spent vs estimated
- Note recurring patterns in Terraria code

### For Team Coordination
- Update `TASK_STATUS.md` religiously
- Commit frequently with descriptive messages
- Tag commits with task IDs: `[M4-NPC-5]`, `[M1.4-BG-3]`, etc.
- Push branches to remote for visibility

---

## ⚠️ Critical Reminders

### Always Do This
✅ Read `agents.md` golden rule before each new task  
✅ Check `TASK_STATUS.md` before starting  
✅ Verify pixel-identical output for visual work  
✅ Document all non-obvious decisions  
✅ Use consistent naming after C# originals  
✅ Test thoroughly before committing  

### Never Do This  
❌ Work on already-assigned tasks  
❌ Guess behavior without reading C#  
❌ Optimize before correctness is verified  
❌ Create conflicting implementations  
❌ Skip tests or benchmarks  
❌ Leave tasks half-finished  

---

## 🆘 When Stuck

1. **Technical blocker?**
   - Read more of surrounding C# code
   - Check callers/calleys chain
   - Look for similar implementations
   - Document the question in ANALYSIS.md

2. **Task conflict?**
   - Check `TASK_STATUS.md` timestamps
   - Respect prior claims
   - Choose different task
   - Escalate to human if urgent overlap

3. **Scope unclear?**
   - Re-read `TERRARIA_SYSTEMS_CATALOG.md`
   - Check milestone boundaries in ROADMAP.md
   - Start narrow, expand later if safe

---

## 📞 Final Checklist Before Committing

- [ ] Read agents.md and confirmed understanding
- [ ] Checked TASK_STATUS.md and claimed my task
- [ ] Located corresponding C# implementation
- [ ] Named functions after original C# code
- [ ] Added source reference comments
- [ ] Verified cargo test --workspace passes
- [ ] Tested visual output against original (if applicable)
- [ ] Updated TASK_STATUS.md with completion status
- [ ] Committed with clear message referencing task ID
- [ ] Pushed branch/commit to remote
- [ ] Noted commit hash in TASK_STATUS.md

---

**This document lives in:** `docs/MULTI_AGENT_WORKFLOW.md`  
**Last updated:** 2026-10-29  
**Next review:** when new agents join or conflicts occur
