# W4-c7 merge: Firefly 355->735, LightningBug 358->736 renumber (d37
# tables); Lavafly 654 / Shimmerfly 677 keep; pins 497->501.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep

def edit_file(p, fn):
    src = open(p, encoding="utf8", newline="").read()
    box = [src]
    rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))
    fn(rep, box)
    open(p, "w", encoding="utf8", newline="").write(box[0])
    print(p, "done")

def data(rep, box):
    src = box[0]
    # to_u16 c7 block
    i = src.index("NpcKind::Firefly => 355,")
    print("to block context:", repr(src[i-450:i+220]))
edit_file("crates/tr-app/src/npc/data.rs", data)
