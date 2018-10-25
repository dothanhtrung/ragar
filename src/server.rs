mod food;
mod player;
mod ragarman;
mod virus;

use food::Food;
use player::Player;
use virus::Virus;

extern crate serde;
extern crate serde_json;
extern crate tokio;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate futures;

use serde_json::Value;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};
use tokio::net::UdpSocket;
use tokio::prelude::*;

fn moving_x_y(src: (f32, f32), des: (f32, f32), v: f32) -> (f32, f32) {
    if (des.0 - src.0).abs() <= v && (des.1 - src.1).abs() <= v {
        (des.0 - src.0, des.1 - src.1)
    } else if des.0 == src.0 {
        (0.0, v)
    } else if des.1 == src.1 {
        (v, 0.0)
    } else {
        // distance in 1 tick is v
        // vector direction is (des.x-point.x)/(des.y-point.y)
        let vec0 = des.0 - src.0;
        let vec1 = des.1 - src.1;
        let vec = (vec0 / vec1).abs();
        let mut dy = (v * v / (vec * vec + 1.0)).sqrt();
        let mut dx = vec * dy;

        dx = dx * if vec0 > 0.0 { 1.0 } else { -1.0 };
        dy = dy * if vec1 > 0.0 { 1.0 } else { -1.0 };

        (dx, dy)
    }
}

fn distance(src: (f32, f32), des: (f32, f32)) -> f32 {
    ((des.0 - src.0).powf(2.0) + (des.1 - src.1).powf(2.0)).sqrt()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    food_mass: u32,
    map_size: (f32, f32),
    v: f32,
    max_food: u32,
    virus_number: u32,
}

impl ServerConfig {
    fn new() -> Self {
        ServerConfig {
            food_mass: 50,
            map_size: (2400.0, 1800.0),
            v: 100.0,
            max_food: 1000,
            virus_number: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct GameData {
    players: Vec<Player>,
    food: Vec<Food>,
    viruses: Vec<Virus>,
    total_food: u32,
}

impl GameData {
    fn new(conf: &ServerConfig) -> Self {
        let mut food = Vec::new();
        let initial_food = 100;
        for _ in 0..initial_food {
            food.push(Food::new(conf.food_mass, conf.map_size));
        }
        let mut viruses = Vec::new();
        for _ in 0..conf.virus_number {
            viruses.push(Virus::new(conf.map_size, conf.food_mass));
        }
        GameData {
            players: Vec::new(),
            food,
            viruses,
            total_food: initial_food,
        }
    }
}

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
    conf: ServerConfig,
    gamedata: GameData,
}

impl Server {
    fn new(socket: UdpSocket) -> Self {
        let conf = ServerConfig::new();
        let gamedata = GameData::new(&conf);

        Server {
            socket,
            buf: vec![0; 1024],
            to_send: None,
            conf,
            gamedata,
        }
    }
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            if let Some((size, peer)) = self.to_send {
                let request = String::from_utf8(self.buf[..size].to_vec()).unwrap();
                let request: Value = serde_json::from_str(&request).unwrap();

                let mut action = "";
                if request["action"].is_string() {
                    action = request["action"].as_str().unwrap();
                }
                let mut id = 0;
                if request["id"].is_u64() {
                    id = request["id"].as_u64().unwrap();
                }

                if action == "new" {
                    let mut name = "";
                    if request["name"].is_string() {
                        name = request["name"].as_str().unwrap();
                    }
                    let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let id = id.as_secs() * 1000 + id.subsec_millis() as u64;
                    let player = Player::new(
                        id,
                        name.to_string(),
                        self.conf.map_size,
                        self.conf.food_mass,
                    );
                    let player_json = serde_json::to_string(&player)?;
                    self.gamedata.players.push(player);

                    let response = format!(
                        "{{\"player\": {}, \"map_size\": [{},{}]}}",
                        player_json, self.conf.map_size.0, self.conf.map_size.1
                    );
                    let _ = try_ready!(self.socket.poll_send_to(&response.as_bytes(), &peer));
                } else if action == "disconnect" {
                    let mut i = 0;
                    while i < self.gamedata.players.len() {
                        if id == self.gamedata.players[i].id {
                            self.gamedata.players.remove(i);
                            break;
                        }
                        i += 1;
                    }
                } else {
                    // Eat food
                    for p in &mut self.gamedata.players {
                        for r in &mut p.ragarmen {
                            let mut i = 0;
                            while i < self.gamedata.food.len() {
                                if ((self.gamedata.food[i].pos.0 - r.pos.0).powi(2)
                                    + (self.gamedata.food[i].pos.1 - r.pos.1).powi(2)).sqrt()
                                    < r.radius
                                {
                                    r.gain_mass(self.conf.food_mass);
                                    self.gamedata.food.remove(i);
                                    self.gamedata.total_food -= 1;
                                } else {
                                    i += 1;
                                }
                            }
                        }
                    }

                    // Eat ragarman
                    let mut i = 0;
                    let mut players_len = self.gamedata.players.len();
                    while players_len != 0 && i < (players_len - 1) {
                        let mut j = i + 1;
                        let mut eaten = false;
                        while j < players_len {
                            let (mut p, mut q) = (0, 0);
                            let (mut leni, mut lenj) = (
                                self.gamedata.players[i].ragarmen.len(),
                                self.gamedata.players[j].ragarmen.len(),
                            );
                            while p < leni {
                                let mut eatenp = false;
                                while q < lenj {
                                    let mut eatenq = false;
                                    let r1 = self.gamedata.players[i].ragarmen[p].clone();
                                    let r2 = self.gamedata.players[j].ragarmen[q].clone();
                                    if r1.mass >= 120 * r2.mass / 100 {
                                        let d = distance(r1.pos, r2.pos);
                                        if d + r2.radius - r1.radius <= r2.radius / 4.0 {
                                            self.gamedata.players[i].ragarmen[p].gain_mass(r2.mass);
                                            self.gamedata.players[j].ragarmen.remove(q);
                                            lenj -= 1;
                                            eatenq = true;
                                        }
                                    } else if r2.mass >= 120 * r1.mass / 100 {
                                        let d = distance(r1.pos, r2.pos);
                                        if d + r1.radius - r2.radius <= r1.radius / 4.0 {
                                            self.gamedata.players[j].ragarmen[q].gain_mass(r1.mass);
                                            self.gamedata.players[i].ragarmen.remove(p);
                                            leni -= 1;
                                            eatenp = true;
                                        }
                                    }
                                    if !eatenq {
                                        q += 1;
                                    }
                                }
                                if !eatenp {
                                    p += 1;
                                }
                            }
                            if lenj <= 0 {
                                self.gamedata.players.remove(j);
                                players_len -= 1;
                            } else {
                                j += 1;
                            }

                            if leni <= 0 {
                                self.gamedata.players.remove(i);
                                players_len -= 1;
                                eaten = true;
                                break;
                            }
                        }
                        if !eaten {
                            i += 1;
                        }
                    }

                    // Sort players
                    if self.gamedata.players.len() > 0 {
                        for i in 0..(self.gamedata.players.len() - 1) {
                            for j in i + 1..self.gamedata.players.len() {
                                if self.gamedata.players[i].mass() < self.gamedata.players[j].mass()
                                {
                                    self.gamedata.players.swap(i, j);
                                }
                            }
                        }
                    }

                    if action == "food" {
                        if self.gamedata.total_food < self.conf.max_food {
                            self.gamedata
                                .food
                                .push(Food::new(self.conf.food_mass, self.conf.map_size));
                            self.gamedata.total_food += 1;
                        }
                    }

                    if request["mouse_pos"].is_array() {
                        let arr_mouse_pos = request["mouse_pos"].as_array().unwrap();
                        if arr_mouse_pos[0].is_number() && arr_mouse_pos[1].is_number() {
                            let mouse_pos = (
                                arr_mouse_pos[0].as_f64().unwrap() as f32,
                                arr_mouse_pos[1].as_f64().unwrap() as f32,
                            );
                            for p in &mut self.gamedata.players {
                                if p.id == id {
                                    p.pos = (0.0, 0.0);
                                    let number_ragaman = p.ragarmen.len();
                                    for r in &mut p.ragarmen {
                                        let mut moving = moving_x_y(
                                            r.pos,
                                            mouse_pos,
                                            self.conf.v / (r.mass as f32).sqrt(),
                                        );
                                        if moving.0 > self.conf.map_size.0 - r.pos.0 {
                                            moving.0 = self.conf.map_size.0 - r.pos.0;
                                        } else if moving.0 < 0.0 - r.pos.0 {
                                            moving.0 = 0.0 - r.pos.0;
                                        }

                                        if moving.1 > self.conf.map_size.1 - r.pos.1 {
                                            moving.1 = self.conf.map_size.1 - r.pos.1;
                                        } else if moving.1 < 0.0 - r.pos.1 {
                                            moving.1 = 0.0 - r.pos.1;
                                        }

                                        r.pos.0 += moving.0;
                                        r.pos.1 += moving.1;
                                        p.pos = (
                                            p.pos.0 + r.pos.0 / (number_ragaman as f32),
                                            p.pos.1 + r.pos.1 / (number_ragaman as f32),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    let gamedata = serde_json::to_string(&self.gamedata)?;
                    let _ = try_ready!(self.socket.poll_send_to(&gamedata.as_bytes(), &peer));
                }
                self.to_send = None;
            }

            self.to_send = Some(try_ready!(self.socket.poll_recv_from(&mut self.buf)));
        }
    }
}
fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:12464".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let socket = UdpSocket::bind(&addr).unwrap();
    println!("Listening on: {}", socket.local_addr().unwrap());

    let server = Server::new(socket);
    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));
}
