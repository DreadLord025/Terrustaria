//! Terraria-Rust: Vulkan renderer prototype for a native reimplementation.

mod app;
mod assets;
mod audio;
mod biome;
mod buffs;
mod collision;
mod inv;
mod item;
mod loot_db;
mod map;
mod menu;
mod net;
mod npc;
mod persist;
mod player;
mod projectile;
mod rain;
mod regen;
mod craft;
mod registry;
mod render;
mod spawner;
mod settings;
mod sound;
mod ui;
mod world_sim;

use std::path::PathBuf;

use anyhow::{Context, Result};
use winit::event_loop::{ControlFlow, EventLoop};

// The `App` lives in `app` (the struct, the event loop, the frame and the
// startup assembly); `main` only orchestrates the bootstrap steps.
fn main() -> Result<()> {
    env_logger::Builder::from_env(
        // The Vulkan loader logs a benign registry warning on every start when no
        // Vulkan SDK layer manifests are installed; keep it out of the console.
        env_logger::Env::default().default_filter_or("info,wgpu_hal::vulkan::instance=error"),
    )
    .init();
    // The release build has no console; leave evidence for any panic instead of
    // losing the message when the window closes.
    std::panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let msg = format!("panic: {info}
{backtrace}
");
        let _ = std::fs::write("crash.log", &msg);
        eprintln!("{msg}");
    }));

    let args: Vec<String> = std::env::args().collect();
    // No argument loads our own generated world (`Documents/My Games/Terrustaria`).
    // Passing a path still loads a `.wld` from the original game or a saved `.trwld`.
    let world_path = if let Some(p) = std::env::var_os("TR_WORLD") {
        PathBuf::from(p)
    } else {
        match args.get(1) {
            Some(p) => PathBuf::from(p),
            None => app::terrstaria_world()?,
        }
    };

    log::info!("world: {}", world_path.display());
    let world = app::load_world(&world_path)?;

    let content_dir = app::find_content_dir().context(
        "assets/Content not found: copy the retail Content directory next to the executable",
    )?;
    assets::set_content_dir(content_dir.clone());
    log::info!("content: {}", content_dir.display());
    let assets_dir = content_dir.join("Images");

    app::log_world_diagnostics(&world);

    let (atlas, lookups, atlas_stats) = app::build_atlas(&world, &assets_dir)?;

    app::log_atlas_diagnostics(&atlas, &lookups);

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = app::App::new(world, world_path, atlas, lookups, atlas_stats);
    // `MapHelper.LoadMap`: restore the explored map from the `.map` sidecar.
    app.load_map();
    event_loop.run_app(&mut app)?;
    Ok(())
}
