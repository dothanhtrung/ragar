extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, Color, DrawMode, Point2};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

// #[derive(PartialEq)]
pub struct Food {
    pub pos: Point2,
    pub draw_pos: Point2,
    color: Color,
}

impl Food {
    pub fn new(ragar_pos: Point2, ragar_draw_pos: Point2) -> Self {
        let x = rand::thread_rng().gen_range(0.0, super::MAP_SIZE.0 as f32);
        let y = rand::thread_rng().gen_range(0.0, super::MAP_SIZE.1 as f32);
        Food {
            pos: Point2::new(x, y),
            draw_pos: Point2::new(
                ragar_draw_pos[0] - (ragar_pos[0] - x),
                ragar_draw_pos[1] - (ragar_pos[1] - y),
            ),
            color: Color::new(
                rand::thread_rng().gen_range(0.0, 1.0),
                rand::thread_rng().gen_range(0.0, 1.0),
                rand::thread_rng().gen_range(0.0, 1.0),
                1.0,
            ),
        }
    }

    pub fn update(&mut self, moving: Point2) {
        self.draw_pos[0] -= moving[0];
        self.draw_pos[1] -= moving[1];
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, self.color)?;
        if self.draw_pos[0] >= 0.0
            && self.draw_pos[0] <= super::SCREEN_SIZE.0 as f32
            && self.draw_pos[1] >= 0.0
            && self.draw_pos[1] <= super::SCREEN_SIZE.1 as f32
        {
            graphics::circle(ctx, DrawMode::Fill, self.draw_pos, 5.0, 2.0)?;
        }
        Ok(())
    }
}
