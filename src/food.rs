extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, Color, Mesh, Point2};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

pub struct Food {
    pub pos: Point2,
    draw_pos: Point2,
    color: Color,
}

impl Food {
    pub fn new(ragar_pos: Point2, ragar_draw_pos: Point2, conf: &super::Config) -> Self {
        let x = rand::thread_rng().gen_range(0.0, conf.map_size.0 as f32);
        let y = rand::thread_rng().gen_range(0.0, conf.map_size.1 as f32);
        let draw_pos = Point2::new(
            ragar_draw_pos[0] - (ragar_pos[0] - x),
            ragar_draw_pos[1] - (ragar_pos[1] - y),
        );
        Food {
            pos: Point2::new(x, y),
            draw_pos,
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

    pub fn draw(&mut self, ctx: &mut Context, mesh: &Mesh, conf: &super::Config) -> GameResult<()> {
        graphics::set_color(ctx, self.color)?;
        if self.draw_pos[0] >= 0.0
            && self.draw_pos[0] <= conf.screen_size.0 as f32
            && self.draw_pos[1] >= 0.0
            && self.draw_pos[1] <= conf.screen_size.1 as f32
        {
            graphics::draw(ctx, mesh, self.draw_pos, 0.0)?;
        }
        Ok(())
    }
}
