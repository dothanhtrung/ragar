mod food;
mod ragarman;

extern crate ggez;

use self::ggez::event;
use self::ggez::graphics::{self, DrawMode, Point2};
use self::ggez::timer;
use self::ggez::{Context, GameResult};

use food::Food;
use ragarman::RagarMan;

const FOOD_MASS: u32 = 50;
const SCREEN_SIZE: (u32, u32) = (1024, 768);
const MAP_SIZE: (u32, u32) = (1280, 1024);
const V: f32 = 100.0;

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
    max_food: u32,
}

impl MainState {
    fn new(_ctx: &mut Context, name: String) -> GameResult<MainState> {
        let ragarman = RagarMan::new(name);
        let mut _food = Vec::new();
        for _ in 0..100 {
            _food.push(Food::new(ragarman.pos, ragarman.draw_pos));
        }
        let s = MainState {
            player: ragarman,
            // ragarmen: vec![ragarman],
            food: _food,
            total_food: 100,
            max_food: 5000,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.player.update(_ctx);
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
                self.player.mass += FOOD_MASS;
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
        if tick % 100 == 0 && self.total_food < self.max_food {
            self.food
                .push(Food::new(self.player.pos, self.player.draw_pos));
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
                MAP_SIZE.0 as f32,
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
                MAP_SIZE.1 as f32,
            ]
                .into(),
        )?;
        // south
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            [
                self.player.draw_pos[0] - self.player.pos[0],
                self.player.draw_pos[1] - (self.player.pos[1] - MAP_SIZE.1 as f32),
                MAP_SIZE.0 as f32,
                5.0,
            ]
                .into(),
        )?;
        // east
        graphics::rectangle(
            ctx,
            DrawMode::Fill,
            [
                self.player.draw_pos[0] - (self.player.pos[0] - MAP_SIZE.0 as f32),
                self.player.draw_pos[1] - self.player.pos[1],
                5.0,
                MAP_SIZE.1 as f32,
            ]
                .into(),
        )?;

        graphics::set_background_color(ctx, [1.0, 1.0, 1.0, 1.0].into());
        self.player.draw(ctx)?;
        // for ragarman in &mut self.ragarmen {
        //     ragarman.draw(ctx)?;
        // }
        for f in &mut self.food {
            f.draw(ctx)?;
        }
        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let ctx = &mut ggez::ContextBuilder::new("ragar", "kim tinh")
        .window_setup(ggez::conf::WindowSetup::default().title("Ragar!"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .expect("Failed to build ggez context");

    let name = "Player One";
    let s = &mut MainState::new(ctx, name.to_string()).unwrap();

    event::run(ctx, s).unwrap();
}
