mod food;
mod player;
mod ragarman;
mod virus;

use food::Food;
use player::Player;
use virus::Virus;

extern crate ggez;
extern crate serde;
extern crate serde_json;
extern crate tokio;

#[macro_use]
extern crate serde_derive;

use self::ggez::event;
use self::ggez::graphics::{self, Font, Mesh, MeshBuilder, Point2, Text};
use self::ggez::timer;
use self::ggez::{Context, GameResult};
use serde_json::Value;
use std::env;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::prelude::*;

struct ClientConf {
    screen_size: (u32, u32),
    font: Font,
}

impl ClientConf {
    fn new() -> Self {
        ClientConf {
            screen_size: (1024, 768),
            font: Font::default_font().unwrap(),
        }
    }
}

struct PlayerDraw {
    player: Player,
    name_display: Text,
}

impl PlayerDraw {
    fn new(_ctx: &mut Context, player: Player, font: &Font) -> Self {
        let name = player.name.clone();
        PlayerDraw {
            player,
            name_display: Text::new(_ctx, &name, font).unwrap(),
        }
    }
}

struct MainState {
    remote_addr: SocketAddr,
    // map_size: (f32, f32),
    id: u64,
    cam_pos: (f32, f32),
    players_draw: Vec<PlayerDraw>,
    food: Vec<Food>,
    viruses: Vec<Virus>,
    conf: ClientConf,
    score_board: Vec<Text>,
}

impl MainState {
    fn new(_ctx: &mut Context, conf: ClientConf) -> GameResult<MainState> {
        let remote_addr: SocketAddr = env::args()
            .nth(1)
            .unwrap_or("127.0.0.1:12464".into())
            .parse()
            .unwrap();
        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }.parse()
        .unwrap();
        let socket = UdpSocket::bind(&local_addr).unwrap();

        let mut r = String::new();
        let _ = socket
            .send_dgram("{\"action\": \"new\", \"name\": \"Test\"}", &remote_addr)
            .and_then(|(socket, _)| socket.recv_dgram(vec![0u8; 65_507]))
            .map(|(_, data, len, _)| r = String::from_utf8(data[..len].to_vec()).unwrap())
            .wait();

        let r: Value = serde_json::from_str(&r).unwrap();
        let player: Player = serde_json::from_value(r["player"].clone()).unwrap();
        let cam_pos = player.ragarmen[0].pos;
        let id = player.id;
        // let mut map_size = (0.0, 0.0);
        // if r["map_size"].is_array() {
        //     let arr_map_size = r["map_size"].as_array().unwrap();
        //     map_size = (
        //         arr_map_size[0].as_f64().unwrap() as f32,
        //         arr_map_size[1].as_f64().unwrap() as f32,
        //     );
        // }

        let mut score_board = Vec::new();
        for i in 0..11 {
            score_board.push(Text::new(_ctx, "", &conf.font).unwrap());
        }

        let s = MainState {
            remote_addr,
            id,
            cam_pos,
            players_draw: Vec::new(),
            // map_size,
            food: Vec::new(),
            viruses: Vec::new(),
            conf,
            score_board,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let mut r = String::new();
        let local_addr: SocketAddr = if self.remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }.parse()
        .unwrap();
        let socket = UdpSocket::bind(&local_addr).unwrap();

        let screen_mouse_pos = ggez::mouse::get_position(_ctx).unwrap();
        let mouse_pos = (
            self.cam_pos.0 - self.conf.screen_size.0 as f32 / 2.0 + screen_mouse_pos[0],
            self.cam_pos.1 - self.conf.screen_size.1 as f32 / 2.0 + screen_mouse_pos[1],
        );

        let mut action = "update";
        let tick = timer::get_ticks(_ctx);
        if tick % 100 == 0 {
            action = "food";
        }
        let _ = socket
            .send_dgram(
                format!(
                    "{{\"action\": \"{}\", \"id\": {}, \"mouse_pos\": [{},{}]}}",
                    action, self.id, mouse_pos.0, mouse_pos.1
                ),
                &self.remote_addr,
            ).and_then(|(socket, _)| socket.recv_dgram(vec![0u8; 65_507]))
            .map(|(_, data, len, _)| r = String::from_utf8(data[..len].to_vec()).unwrap())
            .wait();

        let r: Value = serde_json::from_str(&r).unwrap();
        let food: Vec<Food> = serde_json::from_value(r["food"].clone()).unwrap();
        let viruses: Vec<Virus> = serde_json::from_value(r["viruses"].clone()).unwrap();
        self.food = food;
        self.viruses = viruses;

        let players: Vec<Player> = serde_json::from_value(r["players"].clone()).unwrap();
        self.players_draw = Vec::new();
        self.score_board = Vec::new();
        let mut i = 1;
        let mut in_top_10 = false;
        for p in players {
            if i < 10 {
                if self.id == p.id {
                    in_top_10 = true;
                }
                self.score_board
                    .push(Text::new(_ctx, &format!("{}. {}", i, p.name), &self.conf.font).unwrap());
            } else {
                if self.id == p.id && !in_top_10 {
                    self.score_board.push(
                        Text::new(_ctx, &format!("{}. {}", i, p.name), &self.conf.font).unwrap(),
                    );
                }
            }
            self.players_draw
                .push(PlayerDraw::new(_ctx, p, &self.conf.font));
            i += 1;
        }

        for p in &mut self.players_draw {
            if p.player.id == self.id {
                self.cam_pos = p.player.pos;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::set_background_color(ctx, [1.0, 1.0, 1.0, 1.0].into());

        // Draw border
        // let mesh: Mesh = MeshBuilder::new()
        //     .line(
        //         &[
        //             Point2::new(0.0, 0.0),
        //             Point2::new(0.0, self.map_size.1),
        //             Point2::new(self.map_size.0, self.map_size.1),
        //             Point2::new(self.map_size.0, 0.0),
        //             Point2::new(0.0, 0.0),
        //         ],
        //         4.0,
        //     ).build(ctx)
        //     .unwrap();

        // graphics::draw(
        //     ctx,
        //     &mesh,
        //     Point2::new(
        //         self.conf.screen_size.0 as f32 / 2.0 - self.cam_pos.0,
        //         self.conf.screen_size.1 as f32 / 2.0 - self.cam_pos.1,
        //     ),
        //     0.0,
        // )?;

        // Draw players
        for p in &mut self.players_draw {
            p.player
                .draw(ctx, self.cam_pos, self.conf.screen_size, &p.name_display)?;
        }

        // Draw food
        // let mesh = Mesh::new_circle(ctx, DrawMode::Fill, Point2::new(0.0, 0.0), 5.0, 2.0)?;
        for f in &mut self.food {
            f.draw(ctx, self.cam_pos, self.conf.screen_size)?;
        }

        // Draw viruses
        for v in &mut self.viruses {
            v.draw(ctx, self.cam_pos, self.conf.screen_size)?;
        }

        // Draw score board
        // let mesh: Mesh = MeshBuilder::new()
        //     .line(
        //         &[
        //             Point2::new(self.conf.screen_size.0 as f32 - 200.0, 0.0),
        //             Point2::new(self.conf.screen_size.0 as f32 - 200.0, 250.0),
        //             Point2::new(self.conf.screen_size.0 as f32, 250.0),
        //             Point2::new(self.conf.screen_size.0 as f32, 0.0),
        //         ],
        //         4.0,
        //     ).build(ctx)
        //     .unwrap();
        // graphics::draw(ctx, &mesh, Point2::new(0.0, 0.0), 0.0)?;
        if self.players_draw.len() > 0 {
            graphics::set_color(
                ctx,
                [
                    self.players_draw[0].player.color.0,
                    self.players_draw[0].player.color.1,
                    self.players_draw[0].player.color.2,
                    1.0,
                ]
                    .into(),
            );
        }
        
        let mut i = 0;
        for s in &mut self.score_board {
            graphics::draw(
                ctx,
                s,
                Point2::new(
                    self.conf.screen_size.0 as f32 - 200.0,
                    (i * s.height()) as f32,
                ),
                0.0,
            )?;
            i += 1;
        }

        graphics::present(ctx);
        Ok(())
    }

    /// Send a disconnect signal to server to remove player
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        let remote_addr: SocketAddr = env::args()
            .nth(1)
            .unwrap_or("127.0.0.1:12464".into())
            .parse()
            .unwrap();
        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }.parse()
        .unwrap();
        let socket = UdpSocket::bind(&local_addr).unwrap();

        let _ = socket
            .send_dgram(
                format!("{{\"action\": \"disconnect\", \"id\": {}}}", self.id),
                &remote_addr,
            ).wait();
        false
    }
}

pub fn main() {
    let conf = ClientConf::new();

    let ctx = &mut ggez::ContextBuilder::new("ragar", "kim tinh")
        .window_setup(ggez::conf::WindowSetup::default().title("Ragar!"))
        .window_mode(
            ggez::conf::WindowMode::default().dimensions(conf.screen_size.0, conf.screen_size.1),
        ).build()
        .expect("Failed to build ggez context");

    let s = &mut MainState::new(ctx, conf).unwrap();

    event::run(ctx, s).unwrap();
}
