//! `Terraria.Testing.DebugOptions`'s three SHARED statics — the `94
//! DevCommands` sync state (`Testing/DebugOptions.cs:10-14`, W4-g50
//! re-probe). The C# keeps them as process-wide statics; the port rides
//! them on the App (one process = one game, the i51 single-instance fold).
//!
//! The static ctor's defaults (`DebugOptions.cs:75-79`):
//! `Shared_ReportCommandUsage = true`, `Shared_RandomizeProjectileSlots =
//! false`, `Shared_ServerPing = 0`. `enableDebugCommands` itself is the
//! `TR_DEBUG_COMMANDS` env pin (the d85 registry's gate,
//! `DebugCommandProcessor.cs:30-33`) — the port's pin is presence-based
//! (`env::debug_commands`), the W4-d85 lane's shape.
//!
//! Consumers (the census): `Shared_RandomizeProjectileSlots` gates the
//! `NewProjectile` slot-scan randomizer (`Projectile.cs:10831`, default-off
//! — the port's pool scan is ordered, the arm stays a recorded no-op until
//! the randomizer lands); `Shared_ServerPing` feeds
//! `DebugNetworkStream.Latency = ping / 2` (`MessageBuffer.cs:3482`, the
//! test-harness latency stream is unported — the value stores);
//! `Shared_ReportCommandUsage` gates the command-usage chat report
//! (`ToolkitDebugCommands.cs:107`, the processors are unported — the value
//! stores). The `QuickLoad` probe/response arms stay parked (the harness
//! is unported, `Testing/QuickLoad.cs`).

/// The three shared statics with the C# static-ctor defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DebugOptionsShared {
    /// `DebugOptions.Shared_ReportCommandUsage` (`:10`, default TRUE).
    pub report_command_usage: bool,
    /// `DebugOptions.Shared_RandomizeProjectileSlots` (`:12`, default false).
    pub randomize_projectile_slots: bool,
    /// `DebugOptions.Shared_ServerPing` (`:14`, default 0).
    pub server_ping: i32,
}

impl Default for DebugOptionsShared {
    fn default() -> Self {
        Self {
            report_command_usage: true,
            randomize_projectile_slots: false,
            server_ping: 0,
        }
    }
}
