extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, DrawMode, Point2, Text};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

#[derive(Serialize, Deserialize, Clone)]
pub struct RagarMan {
    pub pos: (f32, f32),
    pub radius: f32,
    pub mass: u32,
    pub name: String,
    pub color: (f32, f32, f32),
}

impl RagarMan {
    pub fn new(map_size: (f32, f32), mass: u32, name: String, color: (f32, f32, f32)) -> Self {
        let x = rand::thread_rng().gen_range(0.0, map_size.0);
        let y = rand::thread_rng().gen_range(0.0, map_size.1);
        RagarMan {
            pos: (x, y),
            radius: (mass as f32 / super::std::f32::consts::PI).sqrt(),
            mass,
            name,
            color,
        }
    }

    pub fn gain_mass(&mut self, mass: u32) {
        self.mass += mass;
        self.radius = (self.mass as f32 / super::std::f32::consts::PI).sqrt();
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        cam_pos: (f32, f32),
        screen_size: (u32, u32),
        name_display: &Text,
    ) -> GameResult<()> {
        graphics::set_color(ctx, [self.color.0, self.color.1, self.color.2, 1.0].into())?;

        let draw_pos = (
            self.pos.0 + screen_size.0 as f32 / 2.0 - cam_pos.0,
            self.pos.1 + screen_size.1 as f32 / 2.0 - cam_pos.1,
        );
        if draw_pos.0 + self.radius >= 0.0
            && draw_pos.0 - self.radius <= screen_size.0 as f32
            && draw_pos.1 + self.radius >= 0.0
            && draw_pos.1 - self.radius <= screen_size.1 as f32
        {
            graphics::circle(
                ctx,
                DrawMode::Fill,
                Point2::new(draw_pos.0, draw_pos.1),
                self.radius,
                1.0,
            )?;
        }

        graphics::set_color(ctx, graphics::BLACK)?;
        graphics::draw(ctx, name_display, Point2::new(draw_pos.0, draw_pos.1), 0.0)?;

        Ok(())
    }
}
