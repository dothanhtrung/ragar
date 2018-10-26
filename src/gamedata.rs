use super::Food;
use super::Player;
use super::ServerConfig;
use super::Virus;

fn distance(src: (f32, f32), des: (f32, f32)) -> f32 {
    ((des.0 - src.0).powf(2.0) + (des.1 - src.1).powf(2.0)).sqrt()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameData {
    pub players: Vec<Player>,
    pub food: Vec<Food>,
    viruses: Vec<Virus>,
    pub total_food: u32,
}

impl GameData {
    /// Create a new game
    pub fn new(conf: &ServerConfig) -> Self {
        let mut food = Vec::new();
        let initial_food = 100;
        for _ in 0..initial_food {
            food.push(Food::new(conf.food_mass, conf.map_size));
        }
        let mut viruses = Vec::new();
        for _ in 0..conf.virus_number {
            viruses.push(Virus::new(conf.map_size, conf.food_mass));
        }
        GameData {
            players: Vec::new(),
            food,
            viruses,
            total_food: initial_food,
        }
    }

    /// Remove player out of list
    pub fn remove_player(&mut self, id: u64) {
        let mut i = 0;
        while i < self.players.len() {
            if id == self.players[i].id {
                self.players.remove(i);
                break;
            }
            i += 1;
        }
    }

    /// Sort players from biggest to smallest
    pub fn sort_players_by_mass(&mut self) {
        if self.players.len() > 0 {
            for i in 0..(self.players.len() - 1) {
                for j in i + 1..self.players.len() {
                    if self.players[i].mass() < self.players[j].mass() {
                        self.players.swap(i, j);
                    }
                }
            }
        }
    }

    /// Add more food to map
    pub fn feed(&mut self, food_mass: u32, map_size: (f32, f32)) {
        self.food.push(Food::new(food_mass, map_size));
        self.total_food += 1;
    }

    // Eat everything they can
    pub fn eat(&mut self) {
        // Eat food
        for p in &mut self.players {
            for r in &mut p.ragarmen {
                let mut i = 0;
                while i < self.food.len() {
                    if ((self.food[i].pos.0 - r.pos.0).powi(2)
                        + (self.food[i].pos.1 - r.pos.1).powi(2)).sqrt()
                        < r.radius
                    {
                        r.gain_mass(self.food[i].mass);
                        self.food.remove(i);
                        self.total_food -= 1;
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Eat ragarman
        let mut i = 0;
        let mut players_len = self.players.len();
        while players_len != 0 && i < (players_len - 1) {
            let mut j = i + 1;
            let mut eaten = false;
            while j < players_len {
                let (mut p, mut q) = (0, 0);
                let (mut leni, mut lenj) = (
                    self.players[i].ragarmen.len(),
                    self.players[j].ragarmen.len(),
                );
                while p < leni {
                    let mut eatenp = false;
                    while q < lenj {
                        let mut eatenq = false;
                        let r1 = self.players[i].ragarmen[p].clone();
                        let r2 = self.players[j].ragarmen[q].clone();
                        if r1.mass >= 120 * r2.mass / 100 {
                            let d = distance(r1.pos, r2.pos);
                            if d + r2.radius - r1.radius <= r2.radius / 4.0 {
                                self.players[i].ragarmen[p].gain_mass(r2.mass);
                                self.players[j].ragarmen.remove(q);
                                lenj -= 1;
                                eatenq = true;
                            }
                        } else if r2.mass >= 120 * r1.mass / 100 {
                            let d = distance(r1.pos, r2.pos);
                            if d + r1.radius - r2.radius <= r1.radius / 4.0 {
                                self.players[j].ragarmen[q].gain_mass(r1.mass);
                                self.players[i].ragarmen.remove(p);
                                leni -= 1;
                                eatenp = true;
                            }
                        }
                        if !eatenq {
                            q += 1;
                        }
                    }
                    if !eatenp {
                        p += 1;
                    }
                }
                if lenj <= 0 {
                    self.players.remove(j);
                    players_len -= 1;
                } else {
                    j += 1;
                }

                if leni <= 0 {
                    self.players.remove(i);
                    players_len -= 1;
                    eaten = true;
                    break;
                }
            }
            if !eaten {
                i += 1;
            }
        }
    }
}
