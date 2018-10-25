extern crate ggez;
extern crate rand;

use self::ggez::graphics::{self, DrawMode, Mesh, MeshBuilder, Point2};
use self::ggez::{Context, GameResult};
use self::rand::Rng;

const GREEN_FREEZE: u8 = 1;
const GREEN_ROLL: u8 = 2;
const ORANGE: u8 = 3;

#[derive(Serialize, Deserialize, Clone)]
pub struct Virus {
    pub pos: (f32, f32),
    pub radius: f32,
    pub mass: u32,
    pub gen: u8,
}

impl Virus {
    pub fn new(map_size: (f32, f32), food_mass: u32) -> Self {
        let x = rand::thread_rng().gen_range(0.0, map_size.0);
        let y = rand::thread_rng().gen_range(0.0, map_size.1);
        let mass = 150 * food_mass;
        Virus {
            pos: (x, y),
            radius: (mass as f32 / super::std::f32::consts::PI).sqrt(),
            mass,
            gen: GREEN_FREEZE,
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
    ) -> GameResult<()> {
        graphics::set_color(ctx, [0.0, 1.0, 0.0, 1.0].into())?;

        let draw_pos = (
            self.pos.0 + screen_size.0 as f32 / 2.0 - cam_pos.0,
            self.pos.1 + screen_size.1 as f32 / 2.0 - cam_pos.1,
        );
        if draw_pos.0 + self.radius >= 0.0
            && draw_pos.0 - self.radius <= screen_size.0 as f32
            && draw_pos.1 + self.radius >= 0.0
            && draw_pos.1 - self.radius <= screen_size.1 as f32
        {
            let shorter_r = self.radius * 0.9;
            let mut points: [Point2; 32] = [Point2::new(0.0, 0.0); 32];

            let mut deg: f32 = 0.0;
            for i in 0..16 {
                let rad = deg / 60.0;
                points[2 * i] = Point2::new(self.radius * rad.sin(), self.radius * rad.cos());
                deg += 22.5;
            }

            deg = 22.5 / 2.0;
            for i in 0..16 {
                let rad = deg / 60.0;
                points[2 * i + 1] = Point2::new(shorter_r * rad.sin(), shorter_r * rad.cos());
                deg += 22.5;
            }

            let mesh: Mesh = MeshBuilder::new()
                .polygon(DrawMode::Fill, &points)
                .build(ctx)
                .unwrap();;

            graphics::draw(ctx, &mesh, Point2::new(draw_pos.0, draw_pos.1), 0.0)?;
        }

        Ok(())
    }
}
