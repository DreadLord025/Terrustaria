# player-draw — roadmap row (pending)

- id: player-draw (arm layering + walk-cycle offsets)
- base: 25e5f6d0 (task/player-draw)
- symptom C1: standing — both arms behind the torso (user build 15:22).
- root cause: `Player_0_6` (shirt) was packed 80 px wide, so `arm_stack`'s
  `FrontShirt` piece (C# `PlayerDrawLayers.cs:3704` draws `Players[skinVar, 6]`
  at `compFrontArmFrame`) sampled column 1 (torso art) over the arm; the
  under-skin/under-shirt/shirt-sleeve sheets (`Players[5]/[8]/[13]`,
  `:1348`/`:1357`/`:1359`/`:3702-3703`) were requested by `make_player_assets`
  but absent from `STATIC_SHEET_NAMES`, so the sleeve pieces never drew.
- fix: +`Player_0_5`/`Player_0_8`/`Player_0_13` in the atlas list;
  `Player_0_6` uncropped; tests t1-t4 pin the C# order and offsets.
- symptom C2: head/torso gap on part of the walk frames — NOT reproduced at
  25e5f6d0: t2/t3 pin the C# offsets (head 0, torso/arms +2 on rows 7-9/14-16,
  `Main.cs:66638-66662`, `PlayerDrawLayers.cs:1217-1219`) and they pass.
- verification: `cargo test -p tr-app -- player_draw` (t1-t4 green),
  `cargo build -p tr-app`, `cs-cite` on the touched files.
