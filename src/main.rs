mod food;
mod ragarman;

extern crate ggez;

use self::ggez::event;
use self::ggez::graphics::{self, DrawMode, Mesh, Point2};
use self::ggez::timer;
use self::ggez::{Context, GameResult};

use food::Food;
use ragarman::RagarMan;

pub struct Config {
    food_mass: u32,
    screen_size: (u32, u32),
    map_size: (u32, u32),
    v: f32,
    max_food: u32,
}

impl Config {
    fn new() -> Self {
        Config {
            food_mass: 50,
            screen_size: (1024, 768),
            map_size: (1280, 1024),
            v: 100.0,
            max_food: 5000,
        }
    }
}

fn moving_x_y(src: &Point2, des: &Point2, v: f32) -> Point2 {
    if (des[0] - src[0]).abs() <= v && (des[1] - src[1]).abs() <= v {
        Point2::new(des[0] - src[0], des[1] - src[1])
    } else if des[0] == src[0] {
        Point2::new(0.0, v)
    } else if des[1] == src[1] {
        Point2::new(v, 0.0)
    } else {
        // distance in 1 tick is v
        // vector direction is (des.x-point.x)/(des.y-point.y)
        let vec0 = des[0] - src[0];
        let vec1 = des[1] - src[1];
        let vec = (vec0 / vec1).abs();
        let mut dy = (v * v / (vec * vec + 1.0)).sqrt();
        let mut dx = vec * dy;

        dx = dx * if vec0 > 0.0 { 1.0 } else { -1.0 };
        dy = dy * if vec1 > 0.0 { 1.0 } else { -1.0 };

        Point2::new(dx, dy)
    }
}

struct MainState {
    player: RagarMan,
    // ragarmen: Vec<RagarMan>,
    food: Vec<Food>,
    total_food: u32,
    conf: Config,
}

impl MainState {
    fn new(_ctx: &mut Context, conf: Config, name: String) -> GameResult<MainState> {
        let ragarman = RagarMan::new(&conf, name);
        let mut _food = Vec::new();
        let initial_food = 1000;
        for _ in 0..initial_food {
            _food.push(Food::new(ragarman.pos, ragarman.draw_pos, &conf));
        }
        let s = MainState {
            player: ragarman,
            // ragarmen: vec![ragarman],
            food: _food,
            total_food: initial_food,
            conf,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.player.update(_ctx, &self.conf);
        // for ragarman in &mut self.ragarmen {
        //     ragarman.update(_ctx);
        // }

        let mut food_len = self.food.len();
        let mut i = 0;

        while i < food_len {
            if ((self.food[i].pos[0] - self.player.pos[0]).powi(2)
                + (self.food[i].pos[1] - self.player.pos[1]).powi(2)).sqrt()
                < self.player.radius
            {
                self.player.mass += self.conf.food_mass;
                self.player.radius = (self.player.mass as f32 / std::f32::consts::PI).sqrt();
                self.food.remove(i);
                food_len -= 1;
            } else {
                i += 1;
            }
        }

        for f in &mut self.food {
            f.update(self.player.moving);
        }

        let tick = timer::get_ticks(_ctx);
        if tick % 100 == 0 && self.total_food < self.conf.max_food {
            self.food
                .push(Food::new(self.player.pos, self.player.draw_pos, &self.conf));
            self.total_food += 1;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Draw border
        // north
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            [
                self.player.draw_pos[0] - self.player.pos[0],
                self.player.draw_pos[1] - self.player.pos[1],
                self.conf.map_size.0 as f32,
                5.0,
            ]
                .into(),
        )?;
        // west
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            [
                self.player.draw_pos[0] - self.player.pos[0],
                self.player.draw_pos[1] - self.player.pos[1],
                5.0,
                self.conf.map_size.1 as f32,
            ]
                .into(),
        )?;
        // south
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            [
                self.player.draw_pos[0] - self.player.pos[0],
                self.player.draw_pos[1] - (self.player.pos[1] - self.conf.map_size.1 as f32),
                self.conf.map_size.0 as f32,
                5.0,
            ]
                .into(),
        )?;
        // east
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            [
                self.player.draw_pos[0] - (self.player.pos[0] - self.conf.map_size.0 as f32),
                self.player.draw_pos[1] - self.player.pos[1],
                5.0,
                self.conf.map_size.1 as f32,
            ]
                .into(),
        )?;

        graphics::set_background_color(ctx, [1.0, 1.0, 1.0, 1.0].into());
        self.player.draw(ctx)?;
        // for ragarman in &mut self.ragarmen {
        //     ragarman.draw(ctx)?;
        // }
        let mesh = Mesh::new_circle(ctx, DrawMode::Fill, Point2::new(0.0, 0.0), 5.0, 2.0)?;
        for f in &mut self.food {
            f.draw(ctx, &mesh, &self.conf)?;
        }
        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let conf = Config::new();
    let ctx = &mut ggez::ContextBuilder::new("ragar", "kim tinh")
        .window_setup(ggez::conf::WindowSetup::default().title("Ragar!"))
        .window_mode(
            ggez::conf::WindowMode::default().dimensions(conf.screen_size.0, conf.screen_size.1),
        ).build()
        .expect("Failed to build ggez context");

    let name = "Player One";
    let s = &mut MainState::new(ctx, conf, name.to_string()).unwrap();

    event::run(ctx, s).unwrap();
}
