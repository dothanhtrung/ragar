extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, Color, DrawMode, Font, Point2, Text};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

pub struct RagarMan {
    pub pos: Point2,
    pub radius: f32,
    pub mass: u32,
    name: String,
    color: Color,
}

impl RagarMan {
    pub fn new(name: String) -> Self {
        RagarMan {
            pos: Point2::new(0.0, 0.0),
            radius: (10.0 * super::FOOD_MASS as f32 / super::std::f32::consts::PI).sqrt(),
            mass: 10 * super::FOOD_MASS,
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
        self.pos[0] += (mouse_pos[0] - self.pos[0]) * 0.05;
        self.pos[1] += (mouse_pos[1] - self.pos[1]) * 0.05;
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, [0.7, 0.5, 0.3, 1.0].into())?;
        graphics::circle(ctx, DrawMode::Fill, self.pos, self.radius, 2.0)?;

        let font = Font::default_font().unwrap();
        let name_display = Text::new(ctx, self.name.as_str(), &font).unwrap();
        graphics::set_color(ctx, self.color)?;
        graphics::draw(ctx, &name_display, self.pos, 0.0)?;
        Ok(())
    }
}
