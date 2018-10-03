mod food;
mod ragarman;

extern crate ggez;

use self::ggez::event;
use self::ggez::graphics::{self, Color};
use self::ggez::timer;
use self::ggez::{Context, GameResult};

use food::Food;
use ragarman::RagarMan;

const FOOD_MASS: u32 = 50;
const SCREEN_SIZE: (u32, u32) = (1024, 768);
const MAP_SIZE: (u32, u32) = (5000, 3000);

struct MainState {
    ragarmen: Vec<RagarMan>,
    food: Vec<Food>,
    total_food: u32,
    max_food: u32,
}

impl MainState {
    fn new(_ctx: &mut Context, name: String) -> GameResult<MainState> {
        let mut _food = Vec::new();
        for _ in 0..10 {
            _food.push(Food::new());
        }
        let s = MainState {
            ragarmen: vec![RagarMan::new(name)],
            food: _food,
            total_food: 10,
            max_food: 500,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        for ragarman in &mut self.ragarmen {
            ragarman.update(_ctx);
        }
        for ragarman in &mut self.ragarmen {
            let mut food_len = self.food.len();
            let mut i = 0;
            while i < food_len {
                if ((self.food[i].pos[0] - ragarman.pos[0]).powi(2)
                    + (self.food[i].pos[1] - ragarman.pos[1]).powi(2)).sqrt()
                    < ragarman.radius
                {
                    ragarman.mass += FOOD_MASS;
                    ragarman.radius = (ragarman.mass as f32 / std::f32::consts::PI).sqrt() ;
                    self.food.remove(i);
                    food_len -= 1;
                } else {
                    i += 1;
                }
            }
        }
        let tick = timer::get_ticks(_ctx);
        if tick % 100 == 0 && self.total_food < self.max_food {
            self.food.push(Food::new());
            self.total_food += 1;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::set_background_color(ctx, [1.0,1.0,1.0,1.0].into());
        for ragarman in &mut self.ragarmen {
            ragarman.draw(ctx)?;
        }
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
