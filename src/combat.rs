use specs::{Join, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{AttackAction, HealthStats, Name, Strength, SufferDamage},
    message_log::MessageLog,
};

pub struct AttackActionHandler;

impl<'a> System<'a> for AttackActionHandler {
    type SystemData = (
        WriteStorage<'a, AttackAction>,
        WriteStorage<'a, SufferDamage>,
        Write<'a, MessageLog>,
        ReadStorage<'a, Strength>,
        ReadStorage<'a, HealthStats>,
        ReadStorage<'a, Name>,
    );

    fn run(
        &mut self,
        (mut attack_actions, mut suffer_damage, mut log, strength, health_stats, names): Self::SystemData,
    ) {
        for (strength, action, name) in (&strength, &attack_actions, &names).join() {
            if let Some(target_stats) = health_stats.get(action.target) {
                if target_stats.defense > strength.amt {
                    log.log("Took no damage because defense is greater");
                    continue;
                }
                let target_name = names.get(action.target).unwrap();
                let damage = strength.amt - target_stats.defense;
                log.log(format!(
                    "{} dealt {} damage to {}",
                    name, damage, target_name
                ));
                SufferDamage::new_damage(&mut suffer_damage, action.target, -(damage as i32));
            }
        }
        attack_actions.clear();
    }
}
