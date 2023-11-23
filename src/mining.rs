use crate::{
    components::{
        BreakAction, Breakable, DeathDrop, EntityStats, HealthStats, Name, Position, SufferDamage,
        ToolType,
    },
    items::{ItemSpawner, SpawnType},
    ui::message_log::MessageLog,
};
use specs::{Entities, Entity, Join, LendJoin, ReadStorage, System, Write, WriteStorage};

/// Allows tile to be breakable. The tile must contain a breakable and health stats component.
/// The attacker must contain a strength and have breakactions queued up in their system.
/// This checks the tile is breakable by the entity given certain conditions
pub struct TileDestructionSystem;

impl<'a> System<'a> for TileDestructionSystem {
    type SystemData = (
        WriteStorage<'a, BreakAction>,
        WriteStorage<'a, SufferDamage>,
        Write<'a, MessageLog>,
        ReadStorage<'a, EntityStats>,
        ReadStorage<'a, Breakable>,
        ReadStorage<'a, HealthStats>,
        ReadStorage<'a, Name>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut break_actions,
            mut suffer_damage,
            mut log,
            stats,
            breakable,
            health_stats,
            names,
            entities,
        ): Self::SystemData,
    ) {
        for (stats, action, name) in (&stats, &break_actions, &names).join() {
            if let Some((_, tile_name, target_breakable, target_stats)) =
                (&entities, &names, &breakable, &health_stats)
                    .join()
                    .find(|(e, _, _, _)| *e == action.target)
            {
                if !inventory_contains_tool(&target_breakable.by) {
                    log.log(format!("You do not own the correct tool for this {name}."));
                    continue;
                }
                if target_stats.defense > stats.set.strength {
                    log.log("Took no damage because defense is greater");
                    continue;
                }

                let damage = stats.set.strength - target_stats.defense;
                log.log(format!(
                    "{} dealt {} damage to {}",
                    name.0, damage, tile_name.0
                ));
                SufferDamage::new_damage(&mut suffer_damage, action.target, -(damage as i32));
            }
        }

        break_actions.clear()
    }
}

// TODO: when we get the inventory added check that it contains the tool
fn inventory_contains_tool(tool_type: &ToolType) -> bool {
    match tool_type {
        ToolType::Hand => true,
        _ => false,
    }
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {
                amount: vec![amount],
            };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, HealthStats>,
    );

    fn run(&mut self, (mut damage, mut breakable): Self::SystemData) {
        for (stats, damage) in (&mut breakable, &mut damage).join() {
            let old_hp = stats.hp;
            let damage_dealt = damage.amount.iter().sum::<i32>();

            // Addition is used because damage dealt can be positive or negative
            let new_hp = stats.hp as i32 + damage_dealt;
            stats.hp = if new_hp >= 0 { new_hp as usize } else { 0 };

            println!(
                "Old HP: {} | Damage Dealt: {} | New HP: {}",
                old_hp, damage_dealt, stats.hp
            );
        }

        damage.clear();
    }
}

pub struct RemoveDeadTiles;

impl<'a> System<'a> for RemoveDeadTiles {
    type SystemData = (
        ReadStorage<'a, HealthStats>,
        ReadStorage<'a, Position>,
        Entities<'a>,
        ReadStorage<'a, DeathDrop>,
        Write<'a, ItemSpawner>,
    );

    fn run(&mut self, (breakable, positions, entities, drops, mut item_spawner): Self::SystemData) {
        for (stats, pos, e, maybe_item) in
            (&breakable, &positions, &entities, (&drops).maybe()).join()
        {
            if stats.hp == 0 {
                match entities.delete(e) {
                    Ok(..) => {
                        if let Some(item) = maybe_item {
                            item_spawner.request(item.0.id, SpawnType::OnGround(*pos));
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to clean up {} : {}", e.id(), err);
                    }
                }
            }
        }
    }
}
