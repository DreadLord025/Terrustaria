//! The App's front-end flow — the `Main.menuMode == 888` screen state
//! machine (A5) — and the remaining `impl App` glue (A6): HUD clicks,
//! session tails, settings/presets, network and save. (A7 later moves
//! `App` itself into this module.)

pub(crate) mod clicks;
pub(crate) mod flow;
pub(crate) mod netglue;
pub(crate) mod saveglue;
pub(crate) mod session;
pub(crate) mod settings_ui;
