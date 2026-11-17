# player-draw — C# -> Rust map (pending)

| C# | Rust | note |
| --- | --- | --- |
| `PlayerDrawLayers.cs:1238->1257` (12_Skin_Composite: skin torso, legs, back arm) | `crates/tr-app/src/render/player_layers.rs` `body_layers` (TorsoSkin -> LegsSkin -> BackArmSkin) | pinned by `t1_player_arm_layer_order_matches_csharp` |
| `LegacyPlayerRenderer.cs:194/208/214/239` (12_Skin -> 17_Torso -> 21_Head -> 28_ArmOverItem) | `assemble_layers` (body -> item -> front-arm stack) | pinned by t1 (both directions) |
| `Main.cs:66638-66662` (`OffsetsPlayerHeadgear`, rows 7-9/14-16 = 0) + `PlayerDrawLayers.cs:1217-1219` (composite shift `2 - Y`) | `player_layers.rs` `headgear_offset`/`headgear_composite_shift` + `body_layers`/`arm_stack` shift | pinned by t2/t3: head 0, torso/arms +2 on the zero rows |
| `PlayerDrawLayers.cs:1346-1362` (back stack `Players[5]/[8]/[13]`) + `:3695-3705` (front stack `Players[7]/[8]/[13]/[6]`) | `assets.rs` `STATIC_SHEET_NAMES` (+Player_0_5/0_8/0_13) + `entities.rs` `make_player_assets` (shirt uncropped) | the fix: the sleeve/under-arm sheets now pack; `Player_0_6` full width restores the FrontShirt arm cells |
| `PlayerDrawLayers.cs:3704` (`Players[6]` at `compFrontArmFrame`) | `player_layers.rs` `arm_stack` FrontShirt at the arm cell | was sampling torso column 1 through the 80-px crop |
