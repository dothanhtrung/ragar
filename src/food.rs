extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, Color, DrawMode, Point2};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

// #[derive(PartialEq)]
pub struct Food {
    pub pos: Point2,
    color: Color,
}

impl Food {
    pub fn new() -> Self {
        let x = rand::thread_rng().gen_range(0.0, 800.0);
        let y = rand::thread_rng().gen_range(0.0, 800.0);
        Food {
            pos: Point2::new(x, y),
            color: Color::new(
                rand::thread_rng().gen_range(0.0, 1.0),
                rand::thread_rng().gen_range(0.0, 1.0),
                rand::thread_rng().gen_range(0.0, 1.0),
                1.0,
            ),
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, self.color)?;
        graphics::circle(ctx, DrawMode::Fill, self.pos, 5.0, 2.0)?;
        Ok(())
    }
}
