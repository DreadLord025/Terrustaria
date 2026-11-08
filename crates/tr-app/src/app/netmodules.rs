//! The NetModule system: the `82` envelope registry and the ported module
//! bodies.
//!
//! The envelope is `[u16 length][u8 82][u16 moduleId][module body]`
//! (`Terraria/Net/NetPacket.cs:19-42`), dispatched by `NetManager.Read`
//! (`NetManager.cs:48-64`) to the module registered under that id. The ids
//! are sequential in registration order — `NetworkInitializer.cs:11-25` — so
//! the numbering below is fixed even for the modules the port does not carry
//! yet.
//!
//! The ported set for M6-NET-4: Text (chat), Ping (the RTT diagnostic),
//! Ambience (sky-entity spawn relay) and Particles (the spawn relay, no
//! particle system in the port yet). The remaining ids ride reserved with
//! citations — their bodies land with the systems' multiplayer passes.

use std::time::Instant;

use glam::Vec2;
use tr_core::TILE_PX;

use crate::ambience::{self, AmbienceCtx, SkyEntityType};
use crate::app::{App, NetRole};
use crate::net::{self, C2s, S2c};
use crate::player;

/// The module ids in the C#'s registration order
/// (`Terraria/Initializers/NetworkInitializer.cs:11-25`). The unmapped ones
/// are reserved: 0 NetLiquid (the batched liquid sync — the port's clients
/// simulate liquids locally, M6-NET-5), 4 NetBestiary (needs the
/// npc-netId → credit-id table), 5/7 NetCreativeUnlocks (+Report) and
///! 6/10 NetCreativePowers (+Permissions) — the creative registry's
/// per-player split is the M6-CREATIVE follow-up — and 8 NetTeleportPylon
/// (the pylon system is not ported yet).
pub mod module_id {
    pub const LIQUID: u16 = 0; // NetLiquidModule.cs
    pub const TEXT: u16 = 1; // NetTextModule.cs
    pub const PING: u16 = 2; // NetPingModule.cs
    pub const AMBIENCE: u16 = 3; // NetAmbienceModule.cs
    pub const BESTIARY: u16 = 4; // NetBestiaryModule.cs
    pub const CREATIVE_UNLOCKS: u16 = 5; // NetCreativeUnlocksModule.cs
    pub const CREATIVE_POWERS: u16 = 6; // NetCreativePowersModule.cs
    pub const CREATIVE_UNLOCKS_REPORT: u16 = 7; // NetCreativeUnlocksPlayerReportModule.cs
    pub const TELEPORT_PYLON: u16 = 8; // NetTeleportPylonModule.cs
    pub const PARTICLES: u16 = 9; // NetParticlesModule.cs
    pub const CREATIVE_PERMISSIONS: u16 = 10; // NetCreativePowerPermissionsModule.cs
}

/// `SkyEntityType`'s wire byte — the port's declaration order until the C#'s
/// `Terraria/GameContent/Ambience/SkyEntityType.cs` values are pinned.
pub fn sky_entity_type_byte(ty: SkyEntityType) -> u8 {
    match ty {
        SkyEntityType::BirdsV => 0,
        SkyEntityType::Wyvern => 1,
        SkyEntityType::Airship => 2,
        SkyEntityType::AirBalloon => 3,
        SkyEntityType::Eyeball => 4,
        SkyEntityType::Meteor => 5,
        SkyEntityType::BoneSerpent => 6,
        SkyEntityType::Bats => 7,
        SkyEntityType::Butterflies => 8,
        SkyEntityType::LostKite => 9,
        SkyEntityType::Vulture => 10,
        SkyEntityType::PixiePosse => 11,
        SkyEntityType::Seagulls => 12,
        SkyEntityType::SlimeBalloons => 13,
        SkyEntityType::Gastropods => 14,
        SkyEntityType::Pegasus => 15,
        SkyEntityType::EaterOfSouls => 16,
        SkyEntityType::Crimera => 17,
        SkyEntityType::Hellbats => 18,
    }
}

/// The `SkyKind` → `SkyEntityType` mapping for the relay.
fn sky_kind_to_type(k: crate::ambience::SkyKind) -> SkyEntityType {
    use crate::ambience::SkyKind::*;
    match k {
        Butterflies => SkyEntityType::Butterflies,
        LostKite => SkyEntityType::LostKite,
        Pegasus => SkyEntityType::Pegasus,
        Vulture => SkyEntityType::Vulture,
        PixiePosse => SkyEntityType::PixiePosse,
        BirdsPack => SkyEntityType::BirdsV,
        Seagulls => SkyEntityType::Seagulls,
        Gastropod => SkyEntityType::Gastropods,
        SlimeBalloons => SkyEntityType::SlimeBalloons,
        Bats => SkyEntityType::Bats,
        Hellbats => SkyEntityType::Hellbats,
        Eyeball => SkyEntityType::Eyeball,
        Meteor => SkyEntityType::Meteor,
        EaterOfSouls => SkyEntityType::EaterOfSouls,
        Crimera => SkyEntityType::Crimera,
        BoneSerpent => SkyEntityType::BoneSerpent,
        Airship => SkyEntityType::Airship,
        AirBalloon => SkyEntityType::AirBalloon,
        Wyvern => SkyEntityType::Wyvern,
    }
}

pub fn sky_entity_type_from_byte(v: u8) -> Option<SkyEntityType> {
    Some(match v {
        0 => SkyEntityType::BirdsV,
        1 => SkyEntityType::Wyvern,
        2 => SkyEntityType::Airship,
        3 => SkyEntityType::AirBalloon,
        4 => SkyEntityType::Eyeball,
        5 => SkyEntityType::Meteor,
        6 => SkyEntityType::BoneSerpent,
        7 => SkyEntityType::Bats,
        8 => SkyEntityType::Butterflies,
        9 => SkyEntityType::LostKite,
        10 => SkyEntityType::Vulture,
        11 => SkyEntityType::PixiePosse,
        12 => SkyEntityType::Seagulls,
        13 => SkyEntityType::SlimeBalloons,
        14 => SkyEntityType::Gastropods,
        15 => SkyEntityType::Pegasus,
        16 => SkyEntityType::EaterOfSouls,
        17 => SkyEntityType::Crimera,
        18 => SkyEntityType::Hellbats,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Module bodies
// ---------------------------------------------------------------------------

/// `NetTextModule`'s client→server body (`ChatMessage.cs:29-37`): the
/// command id ("" for plain chat) and the text.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub command: String,
    pub text: String,
}

impl ChatMessage {
    pub fn write_to(&self, w: net::NetWriter) -> net::NetWriter {
        w.str(&self.command).str(&self.text)
    }
    pub fn read_from(r: &mut net::NetReader) -> Option<Self> {
        Some(Self {
            command: r.str()?,
            text: r.str()?,
        })
    }
}

/// `NetTextModule`'s server→client body (`NetTextModule.cs:24-37`): the
/// author's slot (255 = the server), the literal-mode text and the RGB.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerMessage {
    pub author: u8,
    pub text: String,
    pub color: net::Rgb,
}

impl ServerMessage {
    pub fn write_to(&self, w: net::NetWriter) -> net::NetWriter {
        w.u8(self.author)
            .u8(0) // the literal NetworkText mode
            .str(&self.text)
            .rgb(self.color)
    }
    pub fn read_from(r: &mut net::NetReader) -> Option<Self> {
        let author = r.u8()?;
        let mode = r.u8()?;
        let text = r.str()?;
        let color = r.rgb()?;
        if mode != 0 {
            // Keyed/format modes carry a substitution list the port does not
            // resolve; keep the raw text (documented in ANALYSIS).
            log::debug!("netmodule text: unsupported mode {mode}");
        }
        Some(Self {
            author,
            text,
            color,
        })
    }
}

/// `NetPingModule`'s body (`NetPingModule.cs:9-14`): the invoking player's
/// position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ping {
    pub x: f32,
    pub y: f32,
}

impl Ping {
    pub fn write_to(&self, w: net::NetWriter) -> net::NetWriter {
        w.f32(self.x).f32(self.y)
    }
    pub fn read_from(r: &mut net::NetReader) -> Option<Self> {
        Some(Self {
            x: r.f32()?,
            y: r.f32()?,
        })
    }
}

/// `NetAmbienceModule`'s body (`NetAmbienceModule.cs:11-19`): the invoking
/// player, the spawn seed and the sky-entity type. No position — the
/// receiving side derives it from the seed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbienceSpawn {
    pub player: u8,
    pub seed: i32,
    pub kind: u8,
}

impl AmbienceSpawn {
    pub fn write_to(&self, w: net::NetWriter) -> net::NetWriter {
        w.u8(self.player).i32(self.seed).u8(self.kind)
    }
    pub fn read_from(r: &mut net::NetReader) -> Option<Self> {
        Some(Self {
            player: r.u8()?,
            seed: r.i32()?,
            kind: r.u8()?,
        })
    }
}

/// `NetParticlesModule`'s body (`NetParticlesModule.cs:9-15` +
/// `ParticleOrchestraSettings.cs:16-29`): the type, the world position, the
/// movement vector, the per-type info piece and the invoking player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particles {
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub info: i32,
    pub invoker: u8,
}

impl Particles {
    pub fn write_to(&self, w: net::NetWriter) -> net::NetWriter {
        w.u8(self.kind)
            .f32(self.x)
            .f32(self.y)
            .f32(self.vx)
            .f32(self.vy)
            .i32(self.info)
            .u8(self.invoker)
    }
    pub fn read_from(r: &mut net::NetReader) -> Option<Self> {
        Some(Self {
            kind: r.u8()?,
            x: r.f32()?,
            y: r.f32()?,
            vx: r.f32()?,
            vy: r.f32()?,
            info: r.i32()?,
            invoker: r.u8()?,
        })
    }
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

impl App {
    /// Sends a module envelope to the host (the client-side send).
    fn send_module(&self, module: u16, body: &[u8]) {
        if let NetRole::Client(client) = &self.net {
            client.send(&C2s::NetModule {
                module,
                body: body.to_vec(),
            });
        }
    }

    /// Host side: a client's module arrives (`NetManager.Read` on the server).
    /// Text runs the chat command on the host and relays the result, Ping and
    /// Particles rebroadcast, Ambience is refused like the C#'s
    /// `Deserialize` (`NetAmbienceModule.cs:36-39`), unknown ids are ignored
    /// (`NetManager.cs:55-58`).
    pub(crate) fn host_module(
        &mut self,
        server: &net::Server,
        from: u32,
        module: u16,
        body: &[u8],
    ) {
        let mut r = net::NetReader::new(body);
        match module {
            module_id::TEXT => {
                let Some(msg) = ChatMessage::read_from(&mut r) else {
                    return;
                };
                // The C# routes client commands through the server's command
                // processor (`ChatHelper.cs:72-79`); the port runs its own
                // `ChatMonitor::send` with the sender's name and relays the
                // lines it produced.
                let before = self.chat.messages.len();
                let name = server.slot_name(from);
                let mut rng = self.item_rng;
                self.chat.send(&msg.text, &name, &mut rng, &mut self.emotes);
                self.item_rng = rng;
                let new_lines: Vec<(String, [u8; 3])> = self
                    .chat
                    .messages
                    .iter()
                    .take(self.chat.messages.len().saturating_sub(before).max(0))
                    .map(|l| (l.text.clone(), l.color))
                    .collect();
                // `ChatHelper.BroadcastChatMessageAs` broadcasts to everyone
                // (no ignoreClient): the sender sees its own line through the
                // relay like every other peer.
                for (text, color) in new_lines {
                    let sm = ServerMessage {
                        author: from as u8,
                        text,
                        color: net::Rgb(color[0], color[1], color[2]),
                    };
                    server.broadcast(&S2c::NetModule {
                        module: module_id::TEXT,
                        body: sm.encode(),
                    });
                }
            }
            module_id::PING => {
                // The port echoes the ping back to the sender so the client
                // can measure an RTT (the C# uses the module only for its
                // diagnostics UI and skips the sender — deviation).
                if let Some(p) = self.remote_players.get(&from) {
                    let (x, y) = (p.x, p.y);
                    server.send(
                        from,
                        &S2c::NetModule {
                            module,
                            body: Ping { x, y }.encode(),
                        },
                    );
                }
            }
            module_id::PARTICLES => {
                let Some(p) = Particles::read_from(&mut r) else {
                    return;
                };
                server.broadcast_except(
                    &S2c::NetModule {
                        module,
                        body: p.encode(),
                    },
                    from,
                );
            }
            module_id::AMBIENCE => {
                // The C# refuses ambience messages on a server
                // (`NetAmbienceModule.cs:36-39`): the host is the only
                // spawner and broadcasts its own.
            }
            _ => {}
        }
    }

    /// Client side: a module arrives from the host.
    pub(crate) fn client_module(&mut self, module: u16, body: &[u8]) {
        let mut r = net::NetReader::new(body);
        match module {
            module_id::TEXT => {
                let Some(sm) = ServerMessage::read_from(&mut r) else {
                    return;
                };
                self.chat
                    .new_text(sm.text, [sm.color.0, sm.color.1, sm.color.2]);
            }
            module_id::PING => {
                // The echo: the RTT since the request went out.
                if let Some(sent) = self.net_ping_at.take() {
                    self.net_rtt_ms = sent.elapsed().as_millis() as u32;
                }
            }
            module_id::AMBIENCE => {
                let Some(sp) = AmbienceSpawn::read_from(&mut r) else {
                    return;
                };
                let Some(ty) = sky_entity_type_from_byte(sp.kind) else {
                    return;
                };
                // The ctx borrows `self.world` immutably while `spawn` needs
                // `&mut self.ambience` — the struct update syntax is not an
                // option across fields, so borrow the pieces.
                let mut amb = std::mem::take(&mut self.ambience);
                amb.spawn(&self.ambience_ctx(), ty, sp.seed);
                self.ambience = amb;
            }
            module_id::PARTICLES => {
                // The port has no particle orchestration yet; the body
                // round-trips and drops here (documented in ANALYSIS).
            }
            _ => {}
        }
    }

    /// The `AmbienceCtx` the client-side spawn needs — the same construction
    /// as the frame's ambience update (`app/frame.rs`'s builder).
    pub(crate) fn ambience_ctx(&self) -> AmbienceCtx<'_> {
        AmbienceCtx {
            world: &self.world,
            day_time: self.world.day_time,
            eclipse: self.world.eclipse,
            blood_moon: self.world.blood_moon,
            pumpkin_moon: self.world.pumpkin_moon,
            snow_moon: self.world.snow_moon,
            hard_mode: self.world.hard_mode,
            is_raining: self.rain.is_raining(),
            is_storming: self.wind.storming,
            happy_windy_day: self.wind.happy_windy_day,
            too_windy_for_butterflies: self.wind.too_windy_for_butterflies(),
            butterfly_chance: self.world.butterfly_chance,
            wind: self.wind.speed_current,
            zone_desert: self.scene_metrics.zone_desert,
            zone_hallow: self.scene_metrics.zone_hallow,
            zone_beach: self.scene_metrics.zone_beach,
            zone_corrupt: self.scene_metrics.zone_corrupt,
            zone_crimson: self.scene_metrics.zone_crimson,
            zone_jungle: self.scene_metrics.zone_jungle,
            player_center: self.player.as_ref().map_or(self.camera, |p| {
                Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5)
            }),
            player_y: self.player.as_ref().map_or(self.camera.y, |p| p.y),
            day_rate: self.world.day_rate(),
        }
    }

    /// The client's 1 Hz ping request (the RTT diagnostic).
    pub(crate) fn net_module_tick(&mut self) {
        if !matches!(self.net, NetRole::Client(_)) || self.net_state < net::client_state::IN_WORLD {
            return;
        }
        self.net_module_beat = self.net_module_beat.wrapping_add(1);
        if self.net_module_beat % 10 != 0 {
            return; // the net tick runs at 10 Hz → once a second
        }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        self.net_ping_at = Some(Instant::now());
        self.send_module(module_id::PING, &Ping { x: p.x, y: p.y }.encode());
    }

    /// The host's ambience broadcast: every entity the host has spawned and
    /// not yet relayed goes out as one `NetAmbienceModule`
    /// (`AmbienceServer.SpawnForPlayer`'s `BroadcastOrLoopback`,
    /// `AmbienceServer.cs:199-202`).
    pub(crate) fn take_unsent_ambience(&mut self) -> Vec<(SkyEntityType, i32)> {
        let mut out = Vec::new();
        for e in &mut self.ambience.entities {
            if e.net_sent {
                continue;
            }
            e.net_sent = true;
            out.push((sky_kind_to_type(e.kind), e.net_seed));
        }
        out
    }

    /// The client's chat submit (`ChatHelper.SendChatMessageFromClient`):
    /// the message rides the module instead of executing locally.
    pub(crate) fn send_chat_message(&self, command: &str, text: &str) {
        if !matches!(self.net, NetRole::Client(_)) {
            return;
        }
        if let NetRole::Client(client) = &self.net {
            client.send(&C2s::NetModule {
                module: module_id::TEXT,
                body: ChatMessage {
                    command: command.to_string(),
                    text: text.to_string(),
                }
                .encode(),
            });
        }
    }

    /// The host's particle spawn relay (`ParticleOrchestrator
    /// .BroadcastOrRequestParticleSpawn`, `ParticleOrchestrator.cs:79-90`).
    pub(crate) fn broadcast_particles(server: &net::Server, p: Particles) {
        server.broadcast(&S2c::NetModule {
            module: module_id::PARTICLES,
            body: p.encode(),
        });
    }
}

/// A tiny encode helper: the bodies above build a frame-less byte string, so
/// the module writer opens at offset 0 (the envelope is added by the
/// transport) and the body is the frame minus its 3-byte header.
pub(crate) trait ModuleBody {
    fn encode(&self) -> Vec<u8>;
}

impl ModuleBody for ChatMessage {
    fn encode(&self) -> Vec<u8> {
        self.write_to(net::NetWriter::open(0)).finish()[3..].to_vec()
    }
}

impl ModuleBody for ServerMessage {
    fn encode(&self) -> Vec<u8> {
        self.write_to(net::NetWriter::open(0)).finish()[3..].to_vec()
    }
}

impl ModuleBody for Ping {
    fn encode(&self) -> Vec<u8> {
        self.write_to(net::NetWriter::open(0)).finish()[3..].to_vec()
    }
}

impl ModuleBody for AmbienceSpawn {
    fn encode(&self) -> Vec<u8> {
        self.write_to(net::NetWriter::open(0)).finish()[3..].to_vec()
    }
}

impl ModuleBody for Particles {
    fn encode(&self) -> Vec<u8> {
        self.write_to(net::NetWriter::open(0)).finish()[3..].to_vec()
    }
}
