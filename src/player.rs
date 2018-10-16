extern crate ggez;
extern crate rand;

use self::ggez::{Context, GameResult};
use self::rand::Rng;
use super::ragarman::RagarMan;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    name: String,
    // time_alive: String,
    pub ragarmen: Vec<RagarMan>,
    pub color: (f32, f32, f32),
    pub pos: (f32, f32),
}

impl Player {
    pub fn new(id: u64, name: String, map_size: (f32, f32), food_mass: u32) -> Self {
        let mass = 10 * food_mass;
        let color = (
            rand::thread_rng().gen_range(0.0, 1.0),
            rand::thread_rng().gen_range(0.0, 1.0),
            rand::thread_rng().gen_range(0.0, 1.0),
        );
        let ragarman = RagarMan::new(map_size, mass, name.clone(), color);
        let pos = ragarman.pos;
        Player {
            id,
            name,
            ragarmen: vec![ragarman],
            color,
            pos,
        }
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        cam_pos: (f32, f32),
        screen_size: (u32, u32),
    ) -> GameResult<()> {
        for r in &mut self.ragarmen {
            r.draw(ctx, cam_pos, screen_size)?;
        }

        Ok(())
    }
}
