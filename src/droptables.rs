use bracket_random::prelude::RandomNumberGenerator;
use log::{warn, debug};
use specs::{System, ReadStorage, Write, Join};
use weighted_rand::builder::{WalkerTableBuilder, NewBuilder};

use crate::{items::{ItemID, ItemSpawner, ItemQty, SpawnType}, components::{HealthStats, Position, Name}, data_read::ENTITY_DB};


pub struct Drops {
    pub(crate) drop_chance: u32,   // 1 - 100 indicates the chance there is a drop
    pub(crate) loot_table: Vec<Loot>,
}

pub struct Loot {
    pub(crate) id: ItemID,
    pub(crate) qty: DropQty,
    pub(crate) weight: u32,
}

pub enum DropQty {
    Single(usize),
    #[allow(unused)]
    Range{ min: usize, max: usize }
}

const MAX_ITEM_DROPS: u32 = 10;

fn generate_drops(drop_table: &Drops) -> Vec<(ItemID, ItemQty)> {
    let mut drops: Vec<(ItemID, ItemQty)> = vec![];
    let mut rng = RandomNumberGenerator::seeded(99);
    let mut total_drops: u32 = 0;
    let weights: Vec<u32> = drop_table.loot_table.iter().map(|loot| loot.weight).collect();
    let w = WalkerTableBuilder::new(&weights).build();

    let mut roll = rng.range(0, 100);
    while roll < drop_table.drop_chance / 2u32.pow(total_drops) && total_drops < MAX_ITEM_DROPS {
        let idx = w.next();
        let drop = match drop_table.loot_table.get(idx) {
            Some(drop) => { drop }
            None => unreachable!("idx out of bounds of drop table, mismatch between `weights` and `drop_table.loot_table`"),
        };
        let qty = ItemQty(match drop.qty {
            DropQty::Single(n) => n,
            DropQty::Range { min, max } => rng.range(min, max),
        });
        drops.push((drop.id, qty));
        total_drops += 1;
        roll = rng.range(0, 100);
    }

    drops
}

pub struct DeathLootDrop;

impl<'a> System<'a> for DeathLootDrop {
    type SystemData = (
        ReadStorage<'a, HealthStats>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Name>,
        Write<'a, ItemSpawner>,
        );

    fn run(&mut self, (healths, positions, names, mut item_spawner): Self::SystemData) {
        let edb = &ENTITY_DB.lock().unwrap();
        for (pos, _, name) in (&positions, &healths, &names).join().filter(|(_, health, _)| health.hp == 0) {
            // TODO: differeniate between being and world_obj, i mean they should both work the same way...
            debug!("{} in deathloopdrop", name);
            let drop_table = match edb.beings.get_by_name(&name.0) {
                Some(being) => {
                    match &being.loot {
                        Some(dt) => dt,
                        None => {
                            debug!("no loot table for {}, skipping in DeathLootDrop", name);
                            continue
                        }
                    }
                }
                None => {
                    warn!("{} has no being definition.", name);
                    continue
                }
            };
            debug!("{} generating drops", name); 
            let drops = generate_drops(&drop_table);
            debug!("{:?}", drops);
            for drop in drops {
                item_spawner.request_amt(drop.0, SpawnType::OnGround(*pos), drop.1);
            }
        }
    }
}
