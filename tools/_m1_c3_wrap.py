# W4-c3 merge: remove the now-unreachable d34/d38 style-10 wrappers (the
# axis arm owns the machine for all three kinds; dead-code removal).
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

rep("""    /// W4-d34 - the Cursed Skull (34) through the faithful aiStyle-10
    /// machine (`npc_ai_style10::style10_ai`, `NPC.cs:21572-21918`). The
    /// step owns the whole C# state machine and ends with the noTileCollide
    /// integration (`NPC.cs:92538-92540` + `:92605-92607`); the type-34 path
    /// writes no knockBackResist / sound / spawn (the 289/694 tails only),
    /// so the returned `Style10Out` is inert here. `justHit` rides the
    /// strike flag; `netMode` folds 0 (single-player).
    fn update_cursed_skull(&mut self, world: &World, player: &Player, rng: &mut UnifiedRandom) {
        let _ = crate::npc_ai_style10::style10_ai(
            self,
            world,
            player,
            crate::npc_ai_style10::CURSED_SKULL,
            self.just_hit,
            0,
            rng,
        );
    }

""", """    // W4-c3 merge (329 s4): d34's update_cursed_skull wrapper removed -
    // the aiStyle-10 axis arm (update_rng_dust) owns the machine for the
    // canonical CursedSkull with the pre-move latches + drain adapters.

""")

rep("""    /// W4-d38 - the Giant Cursed Skull (289) through the faithful
    /// `npc_ai_style10::style10_ai` (`NPC.cs:21572-21918`, the aiStyle-10
    /// arm). The 289 path: the 600-tick cycle, the distance-banded approach,
    /// the 120-tick charge → 40-tick dash with the tick-20 Shadowflames bolt
    /// (299, 25 dmg, 6 speed, `:21804-21811`). The module owns the
    /// noTileCollide integration; the bolt drains into `wants_hostile_shot`.
    /// The 34/289 paths draw NO `Main.rand` (the only draw is the 694 book
    /// tail's, `:21851`). `justHit` folds onto `hurt_cd > 0` (the module
    /// convention); netMode 0 (single-player).
    fn update_giant_cursed_skull(&mut self, world: &World, player: &Player, rng: &mut UnifiedRandom) {
        let out = crate::npc_ai_style10::style10_ai(
            self,
            world,
            player,
            crate::npc_ai_style10::GIANT_CURSED_SKULL,
            self.hurt_cd > 0.0,
            0,
            rng,
        );
        // The 289 path writes no knockBackResist override and spawns no
        // Water Sphere (those are the 694 book's); the Item8 play is the
        // book's too. The bolt already rode `wants_hostile_shot` in-module.
        let _ = out;
    }

""", """    // W4-c3 merge (329 s4): d38's update_giant_cursed_skull wrapper removed
    // - the aiStyle-10 axis arm owns the machine (the 299 bolt rides
    // wants_hostile_shot in-module, the drains compose at the axis arm).

""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("wrappers removed")
