//! Minimal multiplayer for Terraria-Rust: length-prefixed TCP frames between a host
//! (which owns the world, NPCs, drops and time) and clients (each simulates its own
//! player and asks the host for edits and pickups).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub const VERSION: u32 = 1;

#[derive(Clone, Debug, Default)]
pub struct PlayerNet {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub facing: i32,
    pub selected: u16,
    pub on_ground: bool,
    pub walk_time: f32,
}

#[derive(Clone, Debug)]
pub struct NpcNet {
    pub id: u32,
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub facing: i32,
    pub on_ground: bool,
    pub anim: f32,
    pub hurt_cd: f32,
}

#[derive(Clone, Debug)]
pub struct ItemNet {
    pub id: u32,
    pub item: u16,
    pub count: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
enum ClientToServer {
    Hello {
        name: String,
        password: String,
    },
    Player(PlayerNet),
    Edit {
        x: i32,
        y: i32,
        kind: u8,
        id: u16,
    },
    Pickup {
        drop: u32,
    },
}

#[derive(Clone, Debug)]
pub enum ServerToClient {
    Welcome {
        id: u32,
    },
    World {
        data: Vec<u8>,
    },
    Joined {
        id: u32,
        name: String,
    },
    Left {
        id: u32,
    },
    Player(PlayerNet),
    Time {
        time: f64,
        day: bool,
    },
    Edit {
        x: i32,
        y: i32,
        kind: u8,
        id: u16,
    },
    Npcs(Vec<NpcNet>),
    Items(Vec<ItemNet>),
    Disconnect {
        reason: String,
    },
}

/// Events the host game loop consumes.
#[derive(Debug)]
pub enum ServerEvent {
    Joined {
        id: u32,
        name: String,
    },
    Left {
        id: u32,
    },
    Player(PlayerNet),
    Edit {
        x: i32,
        y: i32,
        kind: u8,
        tile: u16,
    },
    Pickup {
        id: u32,
        drop: u32,
    },
}

/// Events a joining client game loop consumes.
#[derive(Debug)]
pub enum ClientEvent {
    Welcome {
        id: u32,
    },
    World {
        data: Vec<u8>,
    },
    Joined {
        id: u32,
        name: String,
    },
    Left {
        id: u32,
    },
    Player(PlayerNet),
    Time {
        time: f64,
        day: bool,
    },
    Edit {
        x: i32,
        y: i32,
        kind: u8,
        tile: u16,
    },
    Npcs(Vec<NpcNet>),
    Items(Vec<ItemNet>),
    Disconnect {
        reason: String,
    },
}

fn write_frame(w: &mut impl Write, payload: &[u8]) -> std::io::Result<()> {
    w.write_all(&(payload.len() as u32).to_le_bytes())?;
    w.write_all(payload)
}

fn read_frame(r: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_le_bytes(len) as usize;
    if len > 64 << 20 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

struct W(Vec<u8>);

impl W {
    fn new(tag: u8) -> Self {
        Self(vec![tag])
    }
    fn u8(mut self, v: u8) -> Self {
        self.0.push(v);
        self
    }
    fn u16(mut self, v: u16) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u32(mut self, v: u32) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn i32(mut self, v: i32) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn f32(mut self, v: f32) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn f64(mut self, v: f64) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn bool(mut self, v: bool) -> Self {
        self.0.push(v as u8);
        self
    }
    fn str(mut self, s: &str) -> Self {
        let b = s.as_bytes();
        let n = b.len().min(255);
        self.0.push(n as u8);
        self.0.extend_from_slice(&b[..n]);
        self
    }
    fn bytes(mut self, b: &[u8]) -> Self {
        self.0.extend_from_slice(&(b.len() as u32).to_le_bytes());
        self.0.extend_from_slice(b);
        self
    }
    fn player(self, p: &PlayerNet) -> Self {
        self.u32(p.id)
            .str(&p.name)
            .f32(p.x)
            .f32(p.y)
            .f32(p.vx)
            .f32(p.vy)
            .i32(p.hp)
            .i32(p.max_hp)
            .i32(p.facing)
            .u16(p.selected)
            .bool(p.on_ground)
            .f32(p.walk_time)
    }
}

struct R<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> R<'a> {
    fn new(d: &'a [u8]) -> Self {
        Self { d, p: 0 }
    }
    fn u8(&mut self) -> u8 {
        let v = self.d.get(self.p).copied().unwrap_or(0);
        self.p += 1;
        v
    }
    fn u16(&mut self) -> u16 {
        let mut b = [0u8; 2];
        b.copy_from_slice(self.take(2));
        u16::from_le_bytes(b)
    }
    fn u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(self.take(4));
        u32::from_le_bytes(b)
    }
    fn u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(self.take(8));
        u64::from_le_bytes(b)
    }
    fn i32(&mut self) -> i32 {
        self.u32() as i32
    }
    fn f32(&mut self) -> f32 {
        f32::from_bits(self.u32())
    }
    fn f64(&mut self) -> f64 {
        f64::from_bits(self.u64())
    }
    fn bool(&mut self) -> bool {
        self.u8() != 0
    }
    fn str(&mut self) -> String {
        let n = self.u8() as usize;
        String::from_utf8_lossy(self.take(n)).into_owned()
    }
    fn bytes(&mut self) -> Vec<u8> {
        let n = self.u32() as usize;
        self.take(n).to_vec()
    }
    fn take(&mut self, n: usize) -> &'a [u8] {
        let s = self.d.get(self.p..self.p + n).unwrap_or(&[]);
        self.p += n;
        s
    }
    fn player(&mut self) -> PlayerNet {
        PlayerNet {
            id: self.u32(),
            name: self.str(),
            x: self.f32(),
            y: self.f32(),
            vx: self.f32(),
            vy: self.f32(),
            hp: self.i32(),
            max_hp: self.i32(),
            facing: self.i32(),
            selected: self.u16(),
            on_ground: self.bool(),
            walk_time: self.f32(),
        }
    }
}

fn encode_client(msg: &ClientToServer) -> Vec<u8> {
    match msg {
        ClientToServer::Hello { name, password } => {
            W::new(1).u32(VERSION).str(name).str(password).0
        }
        ClientToServer::Player(p) => W::new(2).player(p).0,
        ClientToServer::Edit { x, y, kind, id } => W::new(3).i32(*x).i32(*y).u8(*kind).u16(*id).0,
        ClientToServer::Pickup { drop } => W::new(4).u32(*drop).0,
    }
}

fn decode_client(d: &[u8]) -> Option<ClientToServer> {
    let mut r = R::new(d);
    Some(match r.u8() {
        1 => {
            let _version = r.u32();
            ClientToServer::Hello {
                name: r.str(),
                password: r.str(),
            }
        }
        2 => ClientToServer::Player(r.player()),
        3 => ClientToServer::Edit {
            x: r.i32(),
            y: r.i32(),
            kind: r.u8(),
            id: r.u16(),
        },
        4 => ClientToServer::Pickup { drop: r.u32() },
        _ => return None,
    })
}

fn encode_server(msg: &ServerToClient) -> Vec<u8> {
    match msg {
        ServerToClient::Welcome { id } => W::new(10).u32(*id).0,
        ServerToClient::World { data } => W::new(11).bytes(data).0,
        ServerToClient::Joined { id, name } => W::new(12).u32(*id).str(name).0,
        ServerToClient::Left { id } => W::new(13).u32(*id).0,
        ServerToClient::Player(p) => W::new(14).player(p).0,
        ServerToClient::Time { time, day } => W::new(15).f64(*time).bool(*day).0,
        ServerToClient::Edit { x, y, kind, id } => W::new(16).i32(*x).i32(*y).u8(*kind).u16(*id).0,
        ServerToClient::Npcs(list) => {
            let mut w = W::new(17).u8(list.len().min(255) as u8);
            for n in list.iter().take(255) {
                w = w
                    .u32(n.id)
                    .u16(n.kind)
                    .f32(n.x)
                    .f32(n.y)
                    .f32(n.vx)
                    .f32(n.vy)
                    .i32(n.hp)
                    .i32(n.max_hp)
                    .i32(n.facing)
                    .bool(n.on_ground)
                    .f32(n.anim)
                    .f32(n.hurt_cd);
            }
            w.0
        }
        ServerToClient::Items(list) => {
            let mut w = W::new(18).u16(list.len().min(1024) as u16);
            for it in list.iter().take(1024) {
                w = w.u32(it.id).u16(it.item).u32(it.count).f32(it.x).f32(it.y);
            }
            w.0
        }
        ServerToClient::Disconnect { reason } => W::new(19).str(reason).0,
    }
}

fn decode_server(d: &[u8]) -> Option<ServerToClient> {
    let mut r = R::new(d);
    Some(match r.u8() {
        10 => ServerToClient::Welcome { id: r.u32() },
        11 => ServerToClient::World { data: r.bytes() },
        12 => ServerToClient::Joined {
            id: r.u32(),
            name: r.str(),
        },
        13 => ServerToClient::Left { id: r.u32() },
        14 => ServerToClient::Player(r.player()),
        15 => ServerToClient::Time {
            time: r.f64(),
            day: r.bool(),
        },
        16 => ServerToClient::Edit {
            x: r.i32(),
            y: r.i32(),
            kind: r.u8(),
            id: r.u16(),
        },
        17 => {
            let n = r.u8() as usize;
            let mut list = Vec::with_capacity(n);
            for _ in 0..n {
                list.push(NpcNet {
                    id: r.u32(),
                    kind: r.u16(),
                    x: r.f32(),
                    y: r.f32(),
                    vx: r.f32(),
                    vy: r.f32(),
                    hp: r.i32(),
                    max_hp: r.i32(),
                    facing: r.i32(),
                    on_ground: r.bool(),
                    anim: r.f32(),
                    hurt_cd: r.f32(),
                });
            }
            ServerToClient::Npcs(list)
        }
        18 => {
            let n = r.u16() as usize;
            let mut list = Vec::with_capacity(n);
            for _ in 0..n {
                list.push(ItemNet {
                    id: r.u32(),
                    item: r.u16(),
                    count: r.u32(),
                    x: r.f32(),
                    y: r.f32(),
                });
            }
            ServerToClient::Items(list)
        }
        19 => ServerToClient::Disconnect { reason: r.str() },
        _ => return None,
    })
}

type ClientSenders = Arc<Mutex<HashMap<u32, Sender<Vec<u8>>>>>;

/// Host side: listener thread plus one reader/writer pair per client.
pub struct Server {
    pub events: Receiver<ServerEvent>,
    clients: ClientSenders,
    addr: String,
    map: Vec<u8>,
}

impl Server {
    /// Binds `0.0.0.0:port` so LAN and internet clients can reach the host.
    pub fn start(port: u16, password: String, map: Vec<u8>) -> std::io::Result<Server> {
        let listener = TcpListener::bind(("0.0.0.0", port))?;
        let addr = listener.local_addr()?.to_string();
        let (tx, rx) = mpsc::channel();
        let clients: ClientSenders = Arc::new(Mutex::new(HashMap::new()));
        let clients_accept = clients.clone();
        thread::spawn(move || {
            let mut next_id: u32 = 1;
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let _ = stream.set_nodelay(true);
                let id = next_id;
                next_id += 1;
                let (out_tx, out_rx) = mpsc::channel::<Vec<u8>>();
                clients_accept.lock().unwrap().insert(id, out_tx);
                let Ok(mut reader) = stream.try_clone() else {
                    continue;
                };
                let mut writer = stream;
                let tx = tx.clone();
                let clients = clients_accept.clone();
                thread::spawn(move || {
                    while let Ok(frame) = out_rx.recv() {
                        if write_frame(&mut writer, &frame).is_err() {
                            break;
                        }
                    }
                    let _ = writer.shutdown(std::net::Shutdown::Both);
                });
                let password = password.clone();
                let clients2 = clients.clone();
                thread::spawn(move || {
                    loop {
                        let Ok(frame) = read_frame(&mut reader) else { break };
                        let Some(msg) = decode_client(&frame) else { continue };
                        match msg {
                            ClientToServer::Hello { name, password: pw } => {
                                if pw != password {
                                    let frame = encode_server(&ServerToClient::Disconnect {
                                        reason: "wrong password".into(),
                                    });
                                    if let Some(s) = clients2.lock().unwrap().get(&id) {
                                        let _ = s.send(frame);
                                    }
                                    thread::sleep(std::time::Duration::from_millis(50));
                                    break;
                                }
                                let _ = tx.send(ServerEvent::Joined { id, name });
                            }
                            ClientToServer::Player(mut p) => {
                                p.id = id;
                                let _ = tx.send(ServerEvent::Player(p));
                            }
                            ClientToServer::Edit { x, y, kind, id: tile } => {
                                let _ = tx.send(ServerEvent::Edit { x, y, kind, tile });
                            }
                            ClientToServer::Pickup { drop } => {
                                let _ = tx.send(ServerEvent::Pickup { id, drop });
                            }
                        }
                    }
                    clients.lock().unwrap().remove(&id);
                    let _ = tx.send(ServerEvent::Left { id });
                });
            }
        });
        Ok(Server {
            events: rx,
            clients,
            addr,
            map,
        })
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub fn send(&self, id: u32, msg: &ServerToClient) {
        let frame = encode_server(msg);
        if let Some(s) = self.clients.lock().unwrap().get(&id) {
            let _ = s.send(frame);
        }
    }

    pub fn broadcast(&self, msg: &ServerToClient) {
        let frame = encode_server(msg);
        for s in self.clients.lock().unwrap().values() {
            let _ = s.send(frame.clone());
        }
    }

    /// Sends the welcome, the compressed world and the current clock to a client.
    pub fn welcome(&self, id: u32, time: f64, day: bool) {
        self.send(id, &ServerToClient::Welcome { id });
        self.send(id, &ServerToClient::World { data: self.map.clone() });
        self.send(id, &ServerToClient::Time { time, day });
    }
}

/// Client side: reader/writer threads plus the event stream for the game loop.
pub struct Client {
    pub events: Receiver<ClientEvent>,
    out: Sender<Vec<u8>>,
}

impl Client {
    pub fn connect(addr: &str, name: String, password: String) -> std::io::Result<Client> {
        let stream = TcpStream::connect(addr)?;
        let _ = stream.set_nodelay(true);
        let mut reader = stream.try_clone()?;
        let (out_tx, out_rx) = mpsc::channel::<Vec<u8>>();
        let (ev_tx, ev_rx) = mpsc::channel();
        let mut writer = stream;
        thread::spawn(move || {
            while let Ok(frame) = out_rx.recv() {
                if write_frame(&mut writer, &frame).is_err() {
                    break;
                }
            }
        });
        thread::spawn(move || loop {
            let Ok(frame) = read_frame(&mut reader) else {
                let _ = ev_tx.send(ClientEvent::Disconnect {
                    reason: "connection closed".into(),
                });
                break;
            };
            let Some(msg) = decode_server(&frame) else { continue };
            let ev = match msg {
                ServerToClient::Welcome { id } => ClientEvent::Welcome { id },
                ServerToClient::World { data } => ClientEvent::World { data },
                ServerToClient::Joined { id, name } => ClientEvent::Joined { id, name },
                ServerToClient::Left { id } => ClientEvent::Left { id },
                ServerToClient::Player(p) => ClientEvent::Player(p),
                ServerToClient::Time { time, day } => ClientEvent::Time { time, day },
                ServerToClient::Edit { x, y, kind, id } => ClientEvent::Edit {
                    x,
                    y,
                    kind,
                    tile: id,
                },
                ServerToClient::Npcs(list) => ClientEvent::Npcs(list),
                ServerToClient::Items(list) => ClientEvent::Items(list),
                ServerToClient::Disconnect { reason } => {
                    let _ = ev_tx.send(ClientEvent::Disconnect { reason });
                    break;
                }
            };
            if ev_tx.send(ev).is_err() {
                break;
            }
        });
        let _ = out_tx.send(encode_client(&ClientToServer::Hello { name, password }));
        Ok(Client {
            events: ev_rx,
            out: out_tx,
        })
    }

    pub fn send_player(&self, p: &PlayerNet) {
        let _ = self
            .out
            .send(encode_client(&ClientToServer::Player(p.clone())));
    }

    pub fn send_edit(&self, x: i32, y: i32, kind: u8, id: u16) {
        let _ = self
            .out
            .send(encode_client(&ClientToServer::Edit { x, y, kind, id }));
    }

    pub fn send_pickup(&self, drop: u32) {
        let _ = self.out.send(encode_client(&ClientToServer::Pickup { drop }));
    }
}
