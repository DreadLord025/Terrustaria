# W4-c9 merge npc_ai_critter.rs: the superset module - the gold rows +
# the faithful TownCritter walk-exit (300+Next(300)-Next(100), the C#
# :53817-53831 re-read) + the idle localAI[3] countdown + the GoldMouse
# idle halve. The d38 module is the base (keep-one per 349 s6b); the c9
# semantic deltas fold in.
import sys, os, re
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _m1_rep import make_rep
p = "crates/tr-app/src/npc_ai_critter.rs"
src = open(p, encoding="utf8", newline="").read()
box = [src]
rep = make_rep(lambda: box[0], lambda v: box.__setitem__(0, v))

# row(): the gold rows join (the c10 superset shape)
rep("""        // `:53902-53906` - flag8 (the Mouse): cap 2, accel 1; NO hop boost.
        NpcKind::Mouse => (2.0, 1.0, false),
        // The bunnies: the default cap 1 / accel 0.07, the TownCritter boost.
        NpcKind::BunnySlimed | NpcKind::BunnyXmas => (1.0, 0.07, true),
        _ => unreachable!("the catchable critters only"),
""", """        // `:53902-53906` - flag8 (the Mouse): cap 2, accel 1; NO hop boost.
        // W4-c9 merge: the GoldMouse 447 joins flag8 (`:53451`).
        NpcKind::Mouse | NpcKind::GoldMouse => (2.0, 1.0, false),
        // The bunnies: the default cap 1 / accel 0.07, the TownCritter boost.
        // W4-c9 merge: the GoldBunny 443 + the GoldFrog 445 (flag10's land
        // row is the default pair, `:53865-53866`).
        NpcKind::BunnySlimed | NpcKind::BunnyXmas | NpcKind::GoldBunny | NpcKind::GoldFrog => {
            (1.0, 0.07, true)
        }
        _ => unreachable!("the catchable critters only"),
""")
# idle: the localAI[3] countdown + the GoldMouse halve (the c9 deltas)
rep("""    if npc.ai[0] == 0.0 {
        // `:53695-53712` - the idle X deadband decay (±0.1 → 0).
""", """    if npc.ai[0] == 0.0 {
        // `:53632-53635` - the idle localAI[3] countdown (W4-c9 merge).
        if npc.local_ai[3] > 0.0 {
            npc.local_ai[3] -= 1.0;
        }
        // `:53697-53700` - flag8 (the GoldMouse) halves the idle X first
        // (W4-c9 merge).
        if npc.kind == NpcKind::GoldMouse {
            npc.vx *= 0.5;
        }
        // `:53695-53712` - the idle X deadband decay (±0.1 → 0).
""")
# walk block: the countdown moves FIRST + the faithful TownCritter exit
rep("""        // `:54226-54242` - the stuck-flip: parked 30 ticks, flip + re-walk.
""", """        // `:53815` - the walk countdown (W4-c9 merge: the C#'s own order -
        // the countdown + the exit run BEFORE the ladder/hop/stuck-flip;
        // the drowning/homing skips are the stand-ins, the homeless
        // critter always decrements).
        npc.ai[1] -= 1.0;
        // `:53817-53831` - walk -> idle: 300 + Next(300), then
        // (TownCritter) - Next(100); localAI[3] = 60 (the c9 faithful
        // draw order - the :53824 subtraction; the d38 module's :54289
        // 60+Next(60) was the sit/talk-state exit, a DIFFERENT AI_007 arm).
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 300.0 + rng.next_max(300) as f32;
            npc.ai[1] -= rng.next_max(100) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 60.0;
            npc.net_update = true;
        }
        // `:54226-54242` - the stuck-flip: parked 30 ticks, flip + re-walk.
""")
# drop the old tail countdown (the :54257 rest-arm mis-mapping)
rep("""        // `:54257-54265` - the walk countdown → idle (the C#'s ai[0]==2/11
        // rest arm; the walk's own duration ends via the idle transition).
        npc.ai[1] -= 1.0;
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 60.0 + rng.next_max(60) as f32;
            npc.net_update = true;
        }
    }
""", """        // W4-c9 merge: the old :54257 tail countdown dropped - the
        // faithful :53815-53831 countdown + exit moved to the block head
        // above (the C#'s own order).
    }
""")
open(p, "w", encoding="utf8", newline="").write(box[0])
print("critter module superset done")
