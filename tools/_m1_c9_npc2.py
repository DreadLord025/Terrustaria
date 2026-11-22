# W4-c9 merge npc.rs part 2: axis dedupe + dispatch + find_frame keep-one.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# update_kind_ai: the c9 Shimmerfly arm dup drops
rep("""            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                self.ai_067_snail(world, player, rng)
            }
            NpcKind::Shimmerfly => self.ai_064_shimmerfly(world, player, rng),
""", """            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                self.ai_067_snail(world, player, rng)
            }
            // E8 merge: the Shimmerfly rides c7's unreachable-64 group
            // above (the axis claims it - the c9 kind-gated arm retired).
""")
# d40's gold dispatch stand-ins drop (c9's faithful machines keep)
rep("""            // W4-d40: GoldBird, GoldBunny, GoldButterfly, GoldFrog - the aiStyle machine is UNWIRED
            // (the style73 precedent); the kind arm keeps the match exhaustive.
            NpcKind::GoldBird
            | NpcKind::GoldBunny
            | NpcKind::GoldButterfly
            | NpcKind::GoldFrog => {}
            // W4-d40: GoldGrasshopper ride the family machine.
            NpcKind::GoldGrasshopper => self.update_slime(world, player, dt),
            // W4-d40: GoldMouse, GoldWorm - the aiStyle machine is UNWIRED
            // (the style73 precedent); the kind arm keeps the match exhaustive.
            NpcKind::GoldMouse
            | NpcKind::GoldWorm => {}
""", """            // W4-c9 merge keep-one: d40's gold stand-ins dropped - c9's
            // faithful machines below own the dispatch (the GoldBird
            // aiStyle-24 bird, the GoldGrasshopper aiStyle-1 hop, the
            // GoldMouse/GoldBunny/GoldFrog AI_007 critter subset). The
            // GoldButterfly stays unwired (its case-444 FindFrame is the
            // row-0 pin; the AI lands with its wave); the GoldWorm rides
            // c8's axis-66 unreachable arm above.
            NpcKind::GoldButterfly => {}
""")
# find_frame: the c9 Shimmerfly dup drops (c7's arm above owns it)
rep("""            // W4-c9 - the Shimmerfly's `case 677` (`NPC.cs:72710-72721`):
            // spriteDirection = direction; the DOUBLE counter increment
            // (the plain ++ then the ++ inside the compare), reset at 6.
            // The `++frame.Y >= 5` wrap counts PIXELS, not rows - the
            // vanilla quirk: frame.Y cycles 0..4 px INSIDE row 0 (the
            // sheet's row height is >= 5 px), so the port's row index
            // never leaves 0 (the sub-row offset is below the port's
            // row-granular frame model - documented fold).
            NpcKind::Shimmerfly => {
                self.sprite_dir = self.facing;
                self.frame_counter += 1.0;
                self.frame_counter += 1.0;
                if self.frame_counter >= 6.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                }
            }
""", """            // E8 merge: the c9 Shimmerfly case-677 dup dropped - c7's
            // arm above owns it (the same double-increment + pixel-wrap
            // quirk, NPC.cs:72710-72721).
""")
# d40's gold find_frame row-0 stand-ins drop (c9's faithful cases keep)
rep("""            NpcKind::GoldBird
            | NpcKind::GoldBunny
            | NpcKind::GoldButterfly
            | NpcKind::GoldFrog
            | NpcKind::GoldGrasshopper => {
                self.frame = 0;
            }
            NpcKind::GoldMouse => {
                self.frame = 0;
            }
""", """            // W4-c9 merge keep-one: d40's gold row-0 stand-ins dropped -
            // c9's faithful FindFrame cases above own the 442/443/445/
            // 446/447 machines. The GoldButterfly keeps the row-0 pin.
            NpcKind::GoldButterfly => {
                self.frame = 0;
            }
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("c9 npc.rs part 2 done")
