mod food;
mod player;
mod ragarman;

use food::Food;
use player::Player;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    food_mass: u32,
    map_size: (f32, f32),
    v: f32,
    max_food: u32,
}

impl ServerConfig {
    fn new() -> Self {
        ServerConfig {
            food_mass: 50,
            map_size: (800.0, 600.0),
            v: 100.0,
            max_food: 1000,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct GameData {
    players: Vec<Player>,
    food: Vec<Food>,
    total_food: u32,
}

impl GameData {
    fn new(conf: &ServerConfig) -> Self {
        let mut _food = Vec::new();
        let initial_food = 100;
        for _ in 0..initial_food {
            _food.push(Food::new(conf.map_size));
        }
        GameData {
            players: Vec::new(),
            food: _food,
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

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            for p in &mut self.gamedata.players {
                for r in &mut p.ragarmen {
                    let mut i = 0;
                    while i < self.gamedata.food.len() {
                        if ((self.gamedata.food[i].pos.0 - r.pos.0).powi(2)
                            + (self.gamedata.food[i].pos.1 - r.pos.1).powi(2)).sqrt()
                            < r.radius
                        {
                            r.mass += self.conf.food_mass;
                            r.radius = (r.mass as f32 / std::f32::consts::PI).sqrt();
                            self.gamedata.food.remove(i);
                            self.gamedata.total_food -= 1;
                        } else {
                            i += 1;
                        }
                    }
                }
            }

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
                    if action == "food" {
                        if self.gamedata.total_food < self.conf.max_food {
                            self.gamedata.food.push(Food::new(self.conf.map_size));
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
                                        p.pos = r.pos;
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
    let conf = ServerConfig::new();
    let gamedata = GameData::new(&conf);
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:12464".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let socket = UdpSocket::bind(&addr).unwrap();
    println!("Listening on: {}", socket.local_addr().unwrap());

    let server = Server {
        socket: socket,
        buf: vec![0; 1024],
        to_send: None,
        conf,
        gamedata,
    };

    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));
}
