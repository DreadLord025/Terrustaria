# W4-c7 merge: Firefly 355->735, LightningBug 358->736 (d37 tables);
# Lavafly 654 / Shimmerfly 677 keep. cs_id/net_id/sheets unchanged.
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
    # to_u16: renumber the two contested ids (355 CyanBeetle / 358 Squid
    # own the vanilla slots in the merged roster). The same 4-line block
    # appears in cs_id (vanilla types - UNCHANGED), so anchor on the
    # to_u16 comment.
    rep("""            NpcKind::Firefly => 355,
            NpcKind::LightningBug => 358,
            NpcKind::Lavafly => 654,
            NpcKind::Shimmerfly => 677,
        }
    }
""", """            // E8 merge renumber (d37 tables): 355/358 are d37's
            // CyanBeetle/Squid in the settled roster - the firefly pair
            // rides the port-local 735/736 (cs_id/net_id stay 355/358).
            NpcKind::Firefly => 735,
            NpcKind::LightningBug => 736,
            NpcKind::Lavafly => 654,
            NpcKind::Shimmerfly => 677,
        }
    }
""")
    # from_u16 mirror
    rep("""            355 => NpcKind::Firefly,
            358 => NpcKind::LightningBug,
            654 => NpcKind::Lavafly,
            677 => NpcKind::Shimmerfly,
""", """            // E8 merge renumber: 735/736 (the 355/358 slots are d37's).
            735 => NpcKind::Firefly,
            736 => NpcKind::LightningBug,
            654 => NpcKind::Lavafly,
            677 => NpcKind::Shimmerfly,
""")

def npc(rep, box):
    # render_key: the firefly pair's keys ride the renumbered ids (the
    # vanilla 355/358 are d37's render keys now)
    rep("""            NpcKind::Firefly => 355,
            NpcKind::LightningBug => 358,
            NpcKind::Lavafly => 654,
            NpcKind::Shimmerfly => 677,
""", """            // E8 merge renumber: the 355/358 keys are d37's above; the
            // firefly pair rides 735/736 (injective, the d34 precedent).
            NpcKind::Firefly => 735,
            NpcKind::LightningBug => 736,
            NpcKind::Lavafly => 654,
            NpcKind::Shimmerfly => 677,
""")
    # ALL_KINDS decl join: 497 + 4 = 501
    rep("""    pub(super) const ALL_KINDS: [NpcKind; 497] = [
    pub(super) const ALL_KINDS: [NpcKind; 274] = [
""", """    // + W4-c7's 4 (Firefly 735 / LightningBug 736 - the d37-table
    // renumber, Lavafly 654 / Shimmerfly 677 keep) = 501.
    pub(super) const ALL_KINDS: [NpcKind; 501] = [
""")
    n = box[0].count("assert_eq!(ALL_KINDS.len(), 497);")
    box[0] = box[0].replace("assert_eq!(ALL_KINDS.len(), 497);", "assert_eq!(ALL_KINDS.len(), 501);")
    print(f"ALL_KINDS.len pins re-pinned: {n}")
    # s1_f_3 pin
    rep("""            497,
            "every decodable kind resolves (the wire roster's size - 489 (spine 488 + c3's 694) + W4-c4's 8 (509/581/587/406/465/592/607/615; the FlyingFish/CorruptGoldfish/BloodFeeder dups deduped onto d37/d34, Pufferfish/Orca retire to d43 per research/148), L-B gate)"
""", """            501,
            "every decodable kind resolves (the wire roster's size - 497 (spine 488 + c3's 694 + c4's 8) + W4-c7's 4 fireflies (Firefly 735 / LightningBug 736 renumbered per the d37 tables, Lavafly 654 / Shimmerfly 677 keep), L-B gate)"
""")
    # c7 witness protocol pins: the pair rides 735/736 now
    rep("""            assert_eq!(kind.to_u16(), ty, "{kind:?} keeps its vanilla protocol id");
            assert_eq!(NpcKind::from_u16(ty), Some(kind), "{kind:?} round-trip");
""", """            // E8 merge renumber: the firefly pair rides 735/736 (the
            // d37 tables); Lavafly/Shimmerfly keep their vanilla ids.
            let proto = match kind {
                NpcKind::Firefly => 735,
                NpcKind::LightningBug => 736,
                _ => ty,
            };
            assert_eq!(kind.to_u16(), proto, "{kind:?} protocol id");
            assert_eq!(NpcKind::from_u16(proto), Some(kind), "{kind:?} round-trip");
""")

edit_file("crates/tr-app/src/npc/data.rs", data)
edit_file("crates/tr-app/src/npc.rs", npc)
