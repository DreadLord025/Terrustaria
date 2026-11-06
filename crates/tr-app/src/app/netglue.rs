//! The App's network glue — the event pump, the host/client event handlers,
//! the 10 Hz state tick, and the message builders/relaunch helper behind
//! them. (A6: extracted from `main.rs`.)

use std::collections::HashMap;
use std::path::PathBuf;

use glam::Vec2;
use tr_core::{TILE_PX, World, save};

use crate::app::{App, NetRole};
use crate::item;
use crate::menu;
use crate::net;
use crate::player;
use crate::world_sim::editing::apply_edit;

impl App {
    /// Drains network events without borrowing `self.net` across handlers.
    pub(crate) fn poll_net(&mut self) {
        let mut server_events = Vec::new();
        let mut client_events = Vec::new();
        match &self.net {
            NetRole::Host(server) => {
                while let Ok(ev) = server.events.try_recv() {
                    server_events.push(ev);
                }
            }
            NetRole::Client(client) => {
                while let Ok(ev) = client.events.try_recv() {
                    client_events.push(ev);
                }
            }
            NetRole::Off => {}
        }
        let role = std::mem::replace(&mut self.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            for ev in server_events {
                self.host_event(server, ev);
            }
        }
        if let NetRole::Client(client) = &role {
            for ev in client_events {
                self.client_event(client, ev);
            }
        }
        self.net = role;
    }

    /// Host side: applies client requests and keeps the state broadcast.
    fn host_event(&mut self, server: &net::Server, ev: net::ServerEvent) {
        match ev {
            net::ServerEvent::Joined { id, name } => {
                log::info!("{name} joined (id {id})");
                server.welcome(id, self.world.time, self.world.day_time);
                for p in self.remote_players.values() {
                    server.send(id, &net_player(p));
                }
                self.remote_players.insert(
                    id,
                    net::PlayerNet {
                        id,
                        name,
                        ..Default::default()
                    },
                );
                broadcast_joined(server, id, &self.remote_players);
            }
            net::ServerEvent::Left { id } => {
                self.remote_players.remove(&id);
                broadcast_left(server, id);
            }
            net::ServerEvent::Player(p) => {
                self.remote_players.insert(p.id, p.clone());
                broadcast_player(server, &p);
            }
            net::ServerEvent::Edit { x, y, kind, tile } => {
                log::info!("net edit kind {kind} at {x},{y}");
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
                server.broadcast(&net_edit(x, y, kind, tile));
            }
            net::ServerEvent::Pickup { id, drop } => {
                let center = self
                    .remote_players
                    .get(&id)
                    .map(|p| Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5));
                if let Some(center) = center {
                    if let Some(c) = self.items.get(drop as usize).map(|it| it.center()) {
                        if (c - center).length() < 60.0 {
                            self.items.remove(drop as usize);
                            server.broadcast(&net_items(&self.items));
                        }
                    }
                }
            }
        }
    }

    /// Client side: clock, edits, NPCs, drops and other players come from the host.
    fn client_event(&mut self, _client: &net::Client, ev: net::ClientEvent) {
        match ev {
            net::ClientEvent::Welcome { id } => {
                self.local_id = id;
                log::info!("joined with id {id}");
            }
            net::ClientEvent::World { data } => {
                if self.menu.is_some() {
                    if let Some((addr, pw)) = self.join_target.clone() {
                        relaunch_into_world(&data, &addr, &pw, &self.player_name);
                    }
                }
            }
            net::ClientEvent::Joined { id, name } => {
                self.remote_players
                    .entry(id)
                    .or_insert_with(|| net::PlayerNet {
                        id,
                        name,
                        ..Default::default()
                    });
            }
            net::ClientEvent::Left { id } => {
                self.remote_players.remove(&id);
            }
            net::ClientEvent::Player(p) => {
                self.remote_players.insert(p.id, p);
            }
            net::ClientEvent::Time { time, day } => {
                self.world.time = time;
                self.world.day_time = day;
            }
            net::ClientEvent::Edit { x, y, kind, tile } => {
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
            }
            net::ClientEvent::Npcs(list) => {
                self.npcs = list
                    .iter()
                    .filter_map(|n| npc_from_net(&self.world, n))
                    .collect();
            }
            net::ClientEvent::Items(list) => {
                self.items = list
                    .iter()
                    .map(|it| {
                        let mut d =
                            item::ItemDrop::new(it.item, it.count, it.x, it.y, 0.0, 0.0, 0.0);
                        d.age = 60.0;
                        d
                    })
                    .collect();
            }
            net::ClientEvent::Disconnect { reason } => {
                log::warn!("disconnected: {reason}");
                self.net = NetRole::Off;
                if self.menu.is_none() {
                    let mut m = menu::Menu::new(self.player_name.clone());
                    m.error = Some(format!("Disconnected: {reason}"));
                    self.menu = Some(m);
                }
            }
        }
    }

    /// Sends our player (and, when hosting, the authoritative state) at 10 Hz.
    pub(crate) fn net_tick(&mut self, dt: f32) {
        if matches!(self.net, NetRole::Off) {
            return;
        }
        self.net_timer += dt;
        if self.net_timer < 0.1 {
            return;
        }
        self.net_timer = 0.0;
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let me = net::PlayerNet {
            id: self.local_id,
            name: self.player_name.clone(),
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            hp: p.hp,
            max_hp: p.max_hp,
            facing: p.facing,
            selected: self.selected as u16,
            on_ground: p.on_ground,
            leg_frame: p.leg_frame,
            body_frame: p.body_frame,
        };
        match &self.net {
            NetRole::Host(server) => {
                broadcast_player(server, &me);
                let npcs: Vec<net::NpcNet> = self
                    .npcs
                    .iter()
                    .enumerate()
                    .map(|(i, n)| net::NpcNet {
                        id: i as u32,
                        kind: n.kind.to_u16(),
                        x: n.x,
                        y: n.y,
                        vx: n.vx,
                        vy: n.vy,
                        hp: n.hp,
                        max_hp: n.max_hp,
                        facing: n.facing,
                        on_ground: n.on_ground,
                        anim: n.anim,
                        hurt_cd: n.hurt_cd,
                    })
                    .collect();
                server.broadcast(&net_npcs(npcs));
                server.broadcast(&net_items(&self.items));
                server.broadcast(&net::ServerToClient::Time {
                    time: self.world.time,
                    day: self.world.day_time,
                });
            }
            NetRole::Client(client) => client.send_player(&me),
            NetRole::Off => {}
        }
    }

    /// Client: asks the host to apply an edit (the host echoes it back to everyone).
    pub(crate) fn request_edit(&self, x: i32, y: i32, kind: u8, tile: u16) {
        match &self.net {
            NetRole::Host(server) => server.broadcast(&net_edit(x, y, kind, tile)),
            NetRole::Client(client) => client.send_edit(x, y, kind, tile),
            NetRole::Off => {}
        }
    }
}

/// Compresses a world for the `World` packet (zlib, speed over ratio: the tile array
/// is mostly empty cells and compresses well).
pub(crate) fn compressed_world(world: &World) -> Vec<u8> {
    use std::io::Write;
    let bytes = save::to_bytes(world).unwrap_or_default();
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    let _ = enc.write_all(&bytes);
    enc.finish().unwrap_or_default()
}

fn decompressed_world(data: &[u8]) -> Option<World> {
    use std::io::Read;
    let mut out = Vec::new();
    let mut dec = flate2::read::ZlibDecoder::new(data);
    dec.read_to_end(&mut out).ok()?;
    save::parse(&out).ok()
}

fn remote_world_path() -> PathBuf {
    save::default_world_path("Remote").unwrap_or_else(|| PathBuf::from("Remote.trwld"))
}

/// Joining receives the host world; saving it and relaunching is the simplest way to
/// rebuild every world-derived system (atlas, liquids, lighting) without tearing down
/// the renderer mid-frame. The child process auto-connects through `TR_JOIN`.
fn relaunch_into_world(data: &[u8], addr: &str, password: &str, name: &str) {
    let Some(world) = decompressed_world(data) else {
        log::warn!("received a world that failed to decode");
        return;
    };
    let path = remote_world_path();
    if let Err(e) = save::save(&path, &world) {
        log::warn!("saving the joined world failed: {e:#}");
        return;
    }
    let exe = std::env::current_exe().unwrap_or_default();
    let mut cmd = std::process::Command::new(exe);
    cmd.env("TR_JOIN", addr)
        .env("TR_JOIN_PW", password)
        .env("TR_WORLD", &path)
        .env("TR_NAME", name)
        .env("TR_JOINED", "1");
    match cmd.spawn() {
        Ok(_) => std::process::exit(0),
        Err(e) => log::warn!("relaunching failed: {e:#}"),
    }
}

fn npc_from_net(world: &World, n: &net::NpcNet) -> Option<crate::npc::Npc> {
    let kind = crate::npc::NpcKind::from_u16(n.kind)?;
    let tx = (n.x / TILE_PX) as i32;
    let mut npc = match kind {
        crate::npc::NpcKind::Slime => crate::npc::Npc::spawn_slime(world, tx)?,
        crate::npc::NpcKind::DemonEye | crate::npc::NpcKind::WanderingEye => {
            crate::npc::Npc::spawn_eye(world, tx)?
        }
        crate::npc::NpcKind::Zombie => crate::npc::Npc::spawn_zombie(world, tx)?,
        crate::npc::NpcKind::Guide => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::Merchant
        | crate::npc::NpcKind::Dryad
        | crate::npc::NpcKind::Nurse
        | crate::npc::NpcKind::ArmsDealer
        | crate::npc::NpcKind::Demolitionist
        | crate::npc::NpcKind::Painter => crate::npc::Npc::spawn_guide(world, tx)?,
        crate::npc::NpcKind::Skeleton | crate::npc::NpcKind::UndeadMiner => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        crate::npc::NpcKind::EaterOfSouls | crate::npc::NpcKind::Hornet | crate::npc::NpcKind::MeteorHead => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        // The swimmers: the ctor plants the feet on the column surface —
        // the net state overwrites the position below.
        crate::npc::NpcKind::Goldfish | crate::npc::NpcKind::Piranha | crate::npc::NpcKind::Shark => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        // The worm segments: the net spawns each link standalone (the chain
        // links rebuild through the init — the head; a lone body/tail dies
        // finding no leader — the net-chain sync lands with the netglue
        // worm work).
        crate::npc::NpcKind::DevourerHead
        | crate::npc::NpcKind::DevourerBody
        | crate::npc::NpcKind::DevourerTail
        | crate::npc::NpcKind::GiantWormHead
        | crate::npc::NpcKind::GiantWormBody
        | crate::npc::NpcKind::GiantWormTail
        | crate::npc::NpcKind::EaterOfWorldsHead
        | crate::npc::NpcKind::EaterOfWorldsBody
        | crate::npc::NpcKind::EaterOfWorldsTail => {
            crate::npc::Npc::spawn_cave_walker(world, tx, kind)?
        }
        // The casters/spheres: the ctor only seeds the stats — the net
        // state overwrites the position below (the hover arm's shape).
        crate::npc::NpcKind::GoblinSorcerer
        | crate::npc::NpcKind::Tim
        | crate::npc::NpcKind::DarkCaster
        | crate::npc::NpcKind::FireImp
        | crate::npc::NpcKind::ChaosBall
        | crate::npc::NpcKind::BurningSphere
        | crate::npc::NpcKind::WaterSphere => {
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_hover(world, tx, y, kind)?
        }
        crate::npc::NpcKind::CaveBat
        | crate::npc::NpcKind::JungleBat
        | crate::npc::NpcKind::Hellbat
        | crate::npc::NpcKind::VoodooDemon => {
            // The net spawn hangs the bat at the net Y (the cave air).
            let y = (n.y / TILE_PX) as i32;
            crate::npc::Npc::spawn_cave_flyer(world, tx, y, kind)?
        }
        crate::npc::NpcKind::KingSlime => crate::npc::Npc::spawn_king_slime(world, tx)?,
    };
    npc.x = n.x;
    npc.y = n.y;
    npc.vx = n.vx;
    npc.vy = n.vy;
    npc.hp = n.hp;
    npc.max_hp = n.max_hp;
    npc.facing = n.facing;
    npc.on_ground = n.on_ground;
    npc.anim = n.anim;
    npc.hurt_cd = n.hurt_cd;
    Some(npc)
}

/// Builders for the host-side broadcast messages.
fn net_player(p: &net::PlayerNet) -> net::ServerToClient {
    net::ServerToClient::Player(p.clone())
}

fn net_edit(x: i32, y: i32, kind: u8, tile: u16) -> net::ServerToClient {
    net::ServerToClient::Edit {
        x,
        y,
        kind,
        id: tile,
    }
}

fn net_npcs(list: Vec<net::NpcNet>) -> net::ServerToClient {
    net::ServerToClient::Npcs(list)
}

fn net_items(items: &[item::ItemDrop]) -> net::ServerToClient {
    net::ServerToClient::Items(
        items
            .iter()
            .enumerate()
            .map(|(i, it)| net::ItemNet {
                id: i as u32,
                item: it.kind,
                count: it.count,
                x: it.pos.x,
                y: it.pos.y,
            })
            .collect(),
    )
}

fn broadcast_player(server: &net::Server, p: &net::PlayerNet) {
    server.broadcast(&net_player(p));
}

fn broadcast_left(server: &net::Server, id: u32) {
    server.broadcast(&net::ServerToClient::Left { id });
}

fn broadcast_joined(server: &net::Server, id: u32, players: &HashMap<u32, net::PlayerNet>) {
    let name = players.get(&id).map(|p| p.name.clone()).unwrap_or_default();
    server.broadcast(&net::ServerToClient::Joined { id, name });
}
