# W4-c8 merge: rebuild the interleaved c7/c8 defaults witnesses.
import sys, os, re
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = "crates/tr-app/src/npc.rs"
src = open(p, encoding="utf8", newline="").read()
nl = "\r\n" if "\r\n" in src else "\n"
lines = src.split(nl)

# locate the region: from the c7 witness fn line to the c8 defaults fn close
i7 = next(k for k, l in enumerate(lines) if "fn w4_c7_the_firefly_defaults" in l)
i8 = next(k for k, l in enumerate(lines) if "fn w4_c8_the_crawler_defaults" in l)
# the c7 axis-witness doc start (after the broken c8 defaults body)
iax = next(k for k, l in enumerate(lines) if "W4-c7 — the axis claims aiStyle 64" in l and k > i8)
# the c8 defaults tail starts at the stray kb assert after the axis witness
itail = next(k for k, l in enumerate(lines) if 'assert_eq!(shell.knock_back_resist(), kb, "{kind:?} kb");' in l and k > iax)
# the c8 defaults fn close: the catchItem ladder's closing brace
iclose = next(k for k, l in enumerate(lines) if "assert_eq!(crate::bug_net::catch_item(606, 0.0), 4363);" in l and k > itail)
iclose += 1  # the `    }` line

c7_witness = """    fn w4_c7_the_firefly_defaults_match_the_csharp_rows() {
        // (kind, vanilla type, w, h)
        let rows = [
            (NpcKind::Firefly, 355u16, 10.0, 10.0),
            (NpcKind::LightningBug, 358, 12.0, 12.0),
            (NpcKind::Lavafly, 654, 10.0, 10.0),
            (NpcKind::Shimmerfly, 677, 10.0, 10.0),
        ];
        for (kind, ty, w, h) in rows {
            let shell = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!(kind.cs_id(), ty, "{kind:?} vanilla type");
            assert_eq!(kind.ai_style(), 64, "{kind:?} aiStyle");
            assert_eq!(kind.size(), (w, h), "{kind:?} size");
            assert_eq!(shell.size(), (w, h), "{kind:?} npc size");
            assert_eq!(kind.contact_damage(), 0, "{kind:?} damage");
            assert_eq!(shell.defense(), 0, "{kind:?} defense");
            assert_eq!(kind.max_hp(), 5, "{kind:?} lifeMax");
            assert_eq!(shell.hp, 5, "{kind:?} spawn hp");
            assert_eq!(shell.knock_back_resist(), 1.0, "{kind:?} kb (class default)");
            assert_eq!(shell.value(), 0.0, "{kind:?} value (no row)");
            assert!(kind.no_gravity(), "{kind:?} noGravity");
            assert_eq!(kind.hit_sound(), Some("NPC_Hit_1"), "{kind:?} HitSound");
            assert_eq!(kind.death_sound(), Some("NPC_Killed_1"), "{kind:?} DeathSound");
            // E8 merge renumber: the firefly pair rides 735/736 (the
            // d37 tables); Lavafly/Shimmerfly keep their vanilla ids.
            let proto = match kind {
                NpcKind::Firefly => 735,
                NpcKind::LightningBug => 736,
                _ => ty,
            };
            assert_eq!(kind.to_u16(), proto, "{kind:?} protocol id");
            assert_eq!(NpcKind::from_u16(proto), Some(kind), "{kind:?} round-trip");
        }
        // `Main.npcFrameCount[355/358/654/677]` = 4 (the FindFrame strips:
        // the 7-tick flap + the two dark-pose rows; the 677's pixel-wrap
        // quirk still reads a 4-row sheet, NPC_677 88x110 -> num 27).
        for (kind, _, _, _) in rows {
            assert_eq!(
                crate::render::entities::npc_rows(crate::render::entities::texture_net_id(kind)),
                4,
                "{kind:?} frame rows"
            );
        }
    }
""".split("\n")

c8_head = """    /// W4-c8 - the crawling-critter family's `SetDefaults` rows pin
    /// against the C# (`NPC.cs:13545-13558` / `:13762-13776` /
    /// `:13777-13795` / `:14332-14346` / `:14576-14589` / `:16781-16794`),
    /// through the port's tables - the small-kinds pin's shape. WRITTEN,
    /// NOT RUN (code-only mode).
    #[test]
    fn w4_c8_the_crawler_defaults_match_the_csharp_rows() {
        // (kind, vanilla type, aiStyle, w, h, kb)
        let rows = [
            (NpcKind::Worm, 357u16, 66u8, 10.0, 4.0, 1.0f32),
            (NpcKind::TruffleWorm, 374, 66, 20.0, 10.0, 1.0),
            (NpcKind::TruffleWormDigger, 375, 6, 10.0, 10.0, 0.0),
            (NpcKind::GoldWorm, 448, 66, 10.0, 4.0, 1.0),
            (NpcKind::EnchantedNightcrawler, 484, 66, 10.0, 4.0, 1.0),
            (NpcKind::Grubby, 485, 66, 10.0, 4.0, 1.0),
            (NpcKind::Sluggy, 486, 66, 10.0, 4.0, 1.0),
            (NpcKind::Buggy, 487, 66, 10.0, 4.0, 1.0),
            (NpcKind::Maggot, 606, 66, 10.0, 4.0, 1.0),
        ];
        for (kind, ty, ai, w, h, kb) in rows {
            let shell = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!(kind.cs_id(), ty, "{kind:?} vanilla type");
            assert_eq!(kind.ai_style(), ai, "{kind:?} aiStyle");
            assert_eq!(kind.size(), (w, h), "{kind:?} size");
            assert_eq!(shell.size(), (w, h), "{kind:?} npc size");
            assert_eq!(kind.contact_damage(), 0, "{kind:?} damage");
            assert_eq!(shell.defense(), 0, "{kind:?} defense");
            assert_eq!(kind.max_hp(), 5, "{kind:?} lifeMax");
            assert_eq!(shell.hp, 5, "{kind:?} spawn hp");
""".split("\n")

# the c8 tail (from the kb assert to the fn close) - re-key the protocol
# round-trip for the renumbered trio
tail = lines[itail:iclose+1]
tail_src = nl.join(tail)
old_rt = '            assert_eq!(NpcKind::from_u16(kind.to_u16()), Some(kind), "{kind:?} round-trip");'
new_rt = ('            // E8 merge renumber: the trio rides 737/738/739 (the d37' + nl +
          '            // tables); the gold/grub dups keep d40\'s vanilla ids, Maggot 606.' + nl +
          '            let proto = match kind {' + nl +
          '                NpcKind::Worm => 737,' + nl +
          '                NpcKind::TruffleWorm => 738,' + nl +
          '                NpcKind::TruffleWormDigger => 739,' + nl +
          '                _ => ty,' + nl +
          '            };' + nl +
          '            assert_eq!(kind.to_u16(), proto, "{kind:?} protocol id");' + nl +
          '            assert_eq!(NpcKind::from_u16(proto), Some(kind), "{kind:?} round-trip");')
assert old_rt in tail_src, "round-trip assert not found in tail"
tail_src = tail_src.replace(old_rt, new_rt)

# assemble: [..i7) + c7_witness + blank + c8_head + tail + (iax..itail) axis witness + [iclose+1..)
# the axis witness block is lines[iax:itail] (doc + fn + body, ends with the for-loop close)
axis = lines[iax:itail]
# strip trailing blank lines from axis
while axis and axis[-1].strip() == "":
    axis.pop()
out = lines[:i7] + c7_witness + [""] + c8_head + tail_src.split(nl) + [""] + axis + [""] + lines[iclose+1:]
open(p, "w", encoding="utf8", newline="").write(nl.join(out))
print("witnesses rebuilt")
