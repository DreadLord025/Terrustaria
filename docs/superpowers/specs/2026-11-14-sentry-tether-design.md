# Design: Sentry / Tethered Projectile AI Port

**Date:** 2026-11-14

## Intent

1:1 port of Frost Hydra (`AI_053_FrostHydra`), Ceiling/Hover Turret (`AI_197_CeilingAndHoverTurret`), Shadow Hand (`AI_187_ShadowHand`) projectile AI styles into `crates/tr-app/src/projectile.rs`.

## Approach

**Variant A — direct inline mapping** in `projectile.rs` with `// File.cs:lines` anchors on every ported block. No module refactor before byte-equality evidence.

## Constraints

- Identical math (including C# `Math.Round` banker's rounding).
- Identical update order.
- Identical RNG stream order.
- No behavior changes to `WorldFrameKey` or fixed-step order.
- `docs/MAIN-RULES.md` and `AGENTS.md` apply.

## Implementation Steps

1. `AI_187_ShadowHand` tether physics + unit test.
2. `AI_053_FrostHydra` + `AI_197_CeilingAndHoverTurret` sentry AI.
3. `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids` verification.
4. Docs update (ROADMAP/PORT_MAP/ANALYSIS) + anchors.txt.

## Source Anchors

- `decompiled/Terraria/Projectile.cs:37051-37265` (AI_123)
- `:37266-37491` (AI_053)
- `:40307-40465` (AI_197)
- `:42502-42630` (AI_187)
- `:43724-44398` (AI_015 flails)
- `Projectile.cs:300` SentryLifeTime
- `:528-537` WipableTurret
- `:670-690` TurretShouldPersist
- `:40194-40254` AI_053_HandleSentryNPCTargeting