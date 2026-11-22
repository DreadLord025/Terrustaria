# W4-c3 merge-side repairs in npc.rs: E8 drops (CursedSkull 355 -> d34@267,
# GiantCursedSkull -> d38@289), keep WaterBoltMimic 694; union join repairs;
# axis-10 routing superset (34/289/694 through c3's style10_ai per 329 s4);
# witness re-pins 273 -> 489.
import re as _re, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
def _get(): return src
def _set(v):
    global src
    src = v
rep = make_rep(_get, _set)

# 1. enum: drop the c3 CursedSkull/GiantCursedSkull dup docs+variants
rep("""    /// `NPCID.CursedSkull` (34, `ID/NPCID.cs:10745`) - the dungeon's
    /// floating skull (`aiStyle 10`): 26x28, dmg 35, def 6, 40 HP,
    /// NPCHit2/NPCDeath2, noGravity + noTileCollide, value 150, kb 0.2,
    /// npcSlots 0.75 (`NPC.cs:9150-9165`). WIRED (W4-c3): the axis' 10
    /// arm owns the machine + the noTileCollide integration tail
    /// (`npc_ai_style10::style10_ai`). The protocol id is 355 - the
    /// vanilla 34 slot is the Merchant's port-local band (`to_u16`'s
    /// guard); 354 is W4-c4's in-flight FlyingFish claim.
    CursedSkull,
    /// `NPCID.GiantCursedSkull` (289, `ID/NPCID.cs:11255`) - the
    /// hardmode dungeon's big skull (`aiStyle 10`): 44x44, dmg 60,
    /// def 20, 400 HP, the same flags/value/kb/slots
    /// (`NPC.cs:12753-12768`). WIRED (W4-c3), the 34 seam; the
    /// per-type branch fires the Shadowflames bolt (299) at tick 20
    /// of its 40-tick dash (`NPC.cs:21804-21811`).
    GiantCursedSkull,
    /// `NPCID.WaterBoltMimic`""",
"""    // W4-c3 merge-side E8 drop (329 s3): the CursedSkull/GiantCursedSkull
    // dup variants dropped - d34@267 / d38@289 own the canonical kinds
    // (the enum docs above); c3's machine wires them through the axis.
    /// `NPCID.WaterBoltMimic`""")

# 2. knock_back_resist: drop the shared c3 arm, keep 694
rep("""            // W4-c3 (the style-10 family): 34/289/694 all write
            // `knockBackResist = 0.2f` (`NPC.cs:9163`/`:12766`/`:17750`).
            // The 694's live 0/1/0.2 writes ride `kb_resist_override`
            // (the axis adapter, `Style10Out::knock_back_resist`).
            NpcKind::CursedSkull | NpcKind::GiantCursedSkull | NpcKind::WaterBoltMimic => 0.2,
""", """            // W4-c3: 694 kb 0.2 (`NPC.cs:17750`); its live 0/1/0.2 writes
            // ride `kb_resist_override` (the axis adapter,
            // `Style10Out::knock_back_resist`). The 34/289 rows are
            // d34/d38's above (E8 keep-both).
            NpcKind::WaterBoltMimic => 0.2,
""")

# 3. defense
rep("""            // W4-c3 (the style-10 family): 34 def 6 (`NPC.cs:9155`),
            // 289 def 20 (`:12758`), 694 def 4 (`:17742`).
            NpcKind::CursedSkull => 6,
            NpcKind::GiantCursedSkull => 20,
            NpcKind::WaterBoltMimic => 4,
""", """            // W4-c3: 694 def 4 (`NPC.cs:17742`); the 34/289 defense rows
            // are d34/d38's above (E8 keep-both).
            NpcKind::WaterBoltMimic => 4,
""")

# 4. value
rep("""            // W4-c3 (the style-10 family): 34/289/694 all write
            // `value = 150f` (`NPC.cs:9162`/`:12765`/`:17749`).
            NpcKind::CursedSkull | NpcKind::GiantCursedSkull | NpcKind::WaterBoltMimic => 150.0,
""", """            // W4-c3: 694 value 150 (`NPC.cs:17749`); the 34/289 rows are
            // d34/d38's above (E8 keep-both).
            NpcKind::WaterBoltMimic => 150.0,
""")

# 5. axis join: keep the d35 guarded 9 arm, drop c3's stale plain 9 arm
rep("""            9 if !matches!(self.kind, NpcKind::VileSpit) => self.ai_009_sphere_family(world, player, rng),
            9 => self.ai_009_sphere_family(world, player, rng),
""", """            9 if !matches!(self.kind, NpcKind::VileSpit) => self.ai_009_sphere_family(world, player, rng),
""")

# 6. update_kind_ai: drop d34/d38's now-unreachable machine arms (the
# axis-10 arm above claims all three kinds; superset-module per 329 s4).
rep("""            NpcKind::CursedSkull => self.update_cursed_skull(world, player, rng),
""", """            // W4-c3 merge (329 s4): the CursedSkull rides the axis-10
            // superset module above (the unreachable arm); d34's legacy
            // machine call dropped - update_cursed_skull stays as the
            // pre-axis witness body.
""")
rep("""            NpcKind::GiantCursedSkull => self.update_giant_cursed_skull(world, player, rng),
""", """            // W4-c3 merge (329 s4): the GiantCursedSkull rides the axis-10
            // superset module above (the 299 bolt + tint + drains); d38's
            // legacy machine call dropped.
""")

# 7. find_frame join repair + E8 drop: close DungeonGuardian, drop c3's
# 34/289 module arm (d34's inline case-34 + d38's inline case-289 own the
# canonical arms - byte-faithful to NPC.cs:77055-77066/:72652-72675,
# re-read at this splice), keep the 694 arm.
rep("""            NpcKind::DungeonGuardian => {
                self.frame = 0;
            // W4-c3 (the style-10 family): the Cursed Skull's `case 34`
            // (`NPC.cs:77055-77066`) and the Giant Cursed Skull's `case 289`
            // (`NPC.cs:72652-72675`) ride the faithful wave-B arms
            // (`npc_findframe_b::find_frame_b`, the S1-D-3 FindFrame wave-B
            // module - the 4-tick 3-row strip / the 8-tick cycle + the
            // ai[3]==1 spin bank). The FrameState/FrameInput seam copies the
            // row/counter to/from `Npc` (the module's wiring note).
            NpcKind::CursedSkull | NpcKind::GiantCursedSkull => {
                let ff_kind = if self.kind == NpcKind::CursedSkull {
                    crate::npc_findframe_b::FindFrameKind::CursedSkull
                } else {
                    crate::npc_findframe_b::FindFrameKind::GiantCursedSkull
                };
                let mut s = crate::npc_findframe_b::FrameState {
                    row: self.frame,
                    counter: self.frame_counter,
                    sprite_dir: self.sprite_dir,
                    rotation: self.rotation,
                };
                let i = crate::npc_findframe_b::FrameInput {
                    vx: self.vx,
                    vy: self.vy,
                    direction: self.facing,
                    wet: self.wet,
                    ai: self.ai,
                };
                crate::npc_findframe_b::find_frame_b(ff_kind, &mut s, &i);
                self.frame = s.row;
                self.frame_counter = s.counter;
                self.sprite_dir = s.sprite_dir;
                self.rotation = s.rotation;
            }
            // The Water Bolt Mimic's `case 694`""",
"""            NpcKind::DungeonGuardian => {
                self.frame = 0;
            }
            // W4-c3 merge-side E8 drop (329 s3): the 34/289 find_frame_b
            // arms dropped - d34's inline case-34 (above) and d38's inline
            // case-289 (below) own the canonical arms; both re-read
            // byte-faithful against NPC.cs:77055-77066/:72652-72675 at
            // this splice. The module keeps its CursedSkull/GiantCursedSkull
            // entries for its own witnesses.
            // The Water Bolt Mimic's `case 694`""")

# 8. render_key: drop the c3 dup arms, keep 694
rep("""            // W4-c3 (the style-10 family): the CursedSkull's vanilla 34
            // is the Merchant's protocol slot above, so its hash rides
            // the port-local 355 band (the wyvern renumber's precedent -
            // injective either way); 289/694 are unused by every arm
            // above, so the G6 key stays injective.
            NpcKind::CursedSkull => 355,
            NpcKind::GiantCursedSkull => 289,
            NpcKind::WaterBoltMimic => 694,
""", """            // W4-c3: the WaterBoltMimic's vanilla 694 is an unused render
            // key (injective); the CursedSkull 267 / GiantCursedSkull 289
            // keys are d34/d38's above (E8 drop of the 355 dup).
            NpcKind::WaterBoltMimic => 694,
""")

# 9. ALL_KINDS decl join: keep the 488 pin comment, re-pin to 489 (+694)
rep("""    // Wave-1 splice pin (367 s1): 488 = the pre-wave 270 + W4-d34's 17 +
    // W4-d38's 35 + W4-d37's 46 + W4-d40's 55 (56 minus the CrimsonGoldfish
    // dup - c4 owns @465) + W4-d42's 32 (43 minus the 11 d33-owned dups,
    // d37 tables) + W4-d35's 33 (the 71-140 band, +269 order-preserving
    // renumber 429-461→698-730 executed at THIS splice per 367 s1 E7 -
    // d35 LAST by design; the 290/305/316/326/313/303 branch pins were
    // stale-by-design; the roster grows to 665 by the spine end).
    pub(super) const ALL_KINDS: [NpcKind; 488] = [
    pub(super) const ALL_KINDS: [NpcKind; 273] = [
""", """    // Wave-1 splice pin (367 s1): 489 = the spine-end 488 + W4-c3's 1
    // (WaterBoltMimic 694; its CursedSkull 355 / GiantCursedSkull 289 dup
    // ports dropped per the d37 E8 tables - d34@267/d38@289 own them).
    pub(super) const ALL_KINDS: [NpcKind; 489] = [
""")

# 10. ALL_KINDS rows: drop the two dups, keep 694
rep("""        // W4-c3 (the style-10 family).
        NpcKind::CursedSkull,
        NpcKind::GiantCursedSkull,
        NpcKind::WaterBoltMimic,
""", """        // W4-c3 (the style-10 family; the 34/289 dups dropped per E8).
        NpcKind::WaterBoltMimic,
""")

# 11. all ALL_KINDS.len() pins 488 -> 489
n = src.count("assert_eq!(ALL_KINDS.len(), 488);")
src = src.replace("assert_eq!(ALL_KINDS.len(), 488);", "assert_eq!(ALL_KINDS.len(), 489);")
print(f"ALL_KINDS.len pins re-pinned: {n}")

# 12. s1_f_3 pin join: keep one assert at 489
m = _re.search(_re.escape("            488,".replace("\n", nl)) + r".*?" + _re.escape("            273,".replace("\n", nl)), src, _re.S)
assert m, "s1_f_3 join not found"
i = src.index("            488,")
j = src.index('        );', i)
seg = src[i:j]
# rebuild: single 489 pin + composed message
new_seg = ('            489,' + nl +
           '            "every decodable kind resolves (the wire roster\'s size - the spine-end 488 (367 s1: d34 17 + d38 35 + d37 46 + d40 55 minus the 465 dup + d42 32 minus the 11 d33 dups + d35 33 at 698-730) + W4-c3\'s WaterBoltMimic 694 (the 355/289 dups dropped per the d37 E8 tables), L-B gate)"' + nl)
src = src[:i] + new_seg + src[j:]

# 13. w4_c3 defaults witness: re-key the protocol pin (355 -> 267), keep
# the canonical-variant stat rows (the tables now serve d34/d38's kinds).
rep("""        assert_eq!(NpcKind::CursedSkull.to_u16(), 355);
""", """        assert_eq!(NpcKind::CursedSkull.to_u16(), 267); // d34 canon (E8)
""")
rep("""    /// ids: the CursedSkull's vanilla 34 is the Merchant's slot, so it rides
    /// port-local 355; 289/694 keep their vanilla ids. WRITTEN, NOT RUN
    /// (code-only mode).
""", """    /// ids at the merged roster: CursedSkull rides d34's port-local 267
    /// (the c3 355 dup dropped per E8); 289/694 keep their vanilla ids.
    /// WRITTEN, NOT RUN (code-only mode).
""")

# 14. CS_STYLES: drop the c3 dup rows (d34/d38 rows already in the table)
rep("""        // W4-c3 (the style-10 family, `NPC.cs:9154`/`:12757`/`:17741`).
        (NpcKind::CursedSkull, 10),
        (NpcKind::GiantCursedSkull, 10),
        (NpcKind::WaterBoltMimic, 10),
""", """        // W4-c3 (the style-10 family, `NPC.cs:17741`; the 34/289 rows are
        // d34/d38's above - E8 dedupe).
        (NpcKind::WaterBoltMimic, 10),
""")

open(p, "w", encoding="utf8", newline="").write(src)
print("c3 npc.rs repairs applied")
