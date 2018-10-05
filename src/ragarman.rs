extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, Color, DrawMode, Font, Point2, Text};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

pub struct RagarMan {
    pub pos: Point2,
    pub draw_pos: Point2,
    pub radius: f32,
    pub mass: u32,
    pub moving: Point2,
    name: String,
    color: Color,
}

impl RagarMan {
    pub fn new(name: String) -> Self {
        let x = rand::thread_rng().gen_range(0.0, super::MAP_SIZE.0 as f32);
        let y = rand::thread_rng().gen_range(0.0, super::MAP_SIZE.1 as f32);
        RagarMan {
            pos: Point2::new(x, y),
            draw_pos: Point2::new(
                super::SCREEN_SIZE.0 as f32 / 2.0,
                super::SCREEN_SIZE.1 as f32 / 2.0,
            ),
            radius: (10.0 * super::FOOD_MASS as f32 / super::std::f32::consts::PI).sqrt(),
            mass: 10 * super::FOOD_MASS,
            moving: Point2::new(0.0, 0.0),
            name,
            color: Color::new(
                rand::thread_rng().gen_range(0.0, 1.0),
                rand::thread_rng().gen_range(0.0, 1.0),
                rand::thread_rng().gen_range(0.0, 1.0),
                1.0,
            ),
        }
    }

    pub fn update(&mut self, _ctx: &mut Context) {
        let mouse_pos = ggez::mouse::get_position(_ctx).unwrap();
        self.moving = super::moving_x_y(
            &self.draw_pos,
            &mouse_pos,
            super::V / (self.mass as f32).sqrt(),
        );
    
        if self.moving[0] > super::MAP_SIZE.0 as f32 - self.pos[0] {
            self.moving[0] = super::MAP_SIZE.0 as f32 - self.pos[0];
        } else if self.moving[0] < 0.0 - self.pos[0]{
            self.moving[0] = 0.0 - self.pos[0];
        }

        if self.moving[1] > super::MAP_SIZE.1 as f32 - self.pos[1] {
            self.moving[1] = super::MAP_SIZE.1 as f32 - self.pos[1];
        } else if self.moving[1] < 0.0 - self.pos[1]{
            self.moving[1] = 0.0 - self.pos[1];
        }

        self.pos[0] += self.moving[0];
        self.pos[1] += self.moving[1];
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, [0.7, 0.5, 0.3, 1.0].into())?;
        graphics::circle(ctx, DrawMode::Fill, self.draw_pos, self.radius, 2.0)?;

        let font = Font::default_font().unwrap();
        let name_display = Text::new(ctx, self.name.as_str(), &font).unwrap();
        graphics::set_color(ctx, self.color)?;
        graphics::draw(ctx, &name_display, self.draw_pos, 0.0)?;
        Ok(())
    }
}
