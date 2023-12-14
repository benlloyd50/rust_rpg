use bracket_random::prelude::RandomNumberGenerator;
use serde::{Deserialize, Serialize};

use crate::components::{EntityStats, HealthStats};

pub fn get_random_stats() -> EntityStats {
    let mut rng = RandomNumberGenerator::new();
    let intelligence = rng.range(1, 21);
    let strength = rng.range(1, 21);
    let dexterity = rng.range(1, 21);
    let vitality = rng.range(1, 21);
    let precision = rng.range(1, 21);
    let charisma = rng.range(1, 21);
    EntityStats::from(Stats { intelligence, strength, dexterity, vitality, precision, charisma })
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Stats {
    pub intelligence: usize,
    pub strength: usize,
    pub dexterity: usize,
    pub vitality: usize,
    pub precision: usize,
    pub charisma: usize,
}

impl Stats {
    pub fn zero() -> Self {
        Self { intelligence: 0, strength: 0, dexterity: 0, vitality: 0, precision: 0, charisma: 0 }
    }

    pub fn get_total(&self) -> usize {
        self.intelligence + self.vitality + self.strength + self.dexterity + self.precision + self.charisma
    }

    /// Generates complementary health stats based off the vitality
    pub fn get_health_stats(&self) -> HealthStats {
        HealthStats::new(self.vitality + 10, self.vitality * 2 / 3)
    }
}

pub struct EntityStatsBuilder {
    stats: Stats,
}

impl EntityStatsBuilder {
    /// Initializes stats with all zeroes. Meant to be used with the extension methods `with_*something*`
    pub fn new() -> Self {
        Self { stats: Stats { intelligence: 0, strength: 0, dexterity: 0, vitality: 0, precision: 0, charisma: 0 } }
    }

    pub fn with_intelligence(&mut self, intelligence: usize) -> &mut Self {
        self.stats.intelligence = intelligence;
        self
    }

    pub fn with_strength(&mut self, strength: usize) -> &mut Self {
        self.stats.strength = strength;
        self
    }

    pub fn with_dexterity(&mut self, dexterity: usize) -> &mut Self {
        self.stats.dexterity = dexterity;
        self
    }

    pub fn with_vitality(&mut self, vitality: usize) -> &mut Self {
        self.stats.vitality = vitality;
        self
    }

    pub fn with_precision(&mut self, precision: usize) -> &mut Self {
        self.stats.precision = precision;
        self
    }

    pub fn with_charisma(&mut self, charisma: usize) -> &mut Self {
        self.stats.charisma = charisma;
        self
    }

    pub fn build(&mut self) -> (EntityStats, HealthStats) {
        (EntityStats::from(self.stats), self.stats.get_health_stats())
    }
}
