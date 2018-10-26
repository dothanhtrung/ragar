mod food;
mod gamedata;
mod player;
mod ragarman;
mod virus;

use food::Food;
use gamedata::GameData;
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
                    self.gamedata.remove_player(id);
                } else {
                    // Players eat food and each other
                    self.gamedata.eat();

                    // Sort players
                    self.gamedata.sort_players_by_mass();

                    if action == "food" {
                        if self.gamedata.total_food < self.conf.max_food {
                            self.gamedata.feed(self.conf.food_mass, self.conf.map_size);
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
