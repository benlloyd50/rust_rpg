use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{
        AttackAction, AttackBonus, EntityStats, Equipped, HealAction, HealthStats, Name,
        SufferDamage,
    },
    ui::message_log::MessageLog,
};

pub struct AttackActionHandler;

impl<'a> System<'a> for AttackActionHandler {
    type SystemData = (
        WriteStorage<'a, AttackAction>,
        WriteStorage<'a, SufferDamage>,
        Write<'a, MessageLog>,
        ReadStorage<'a, EntityStats>,
        ReadStorage<'a, HealthStats>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, AttackBonus>,
        ReadStorage<'a, Equipped>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut attack_actions,
            mut suffer_damage,
            mut log,
            stats,
            health_stats,
            names,
            attack_bonus,
            equipped,
            entities,
        ): Self::SystemData,
    ) {
        for (attacker, stats_set, action, name) in
            (&entities, &stats, &attack_actions, &names).join()
        {
            if let Some(target_stats) = health_stats.get(action.target) {
                if target_stats.defense > stats_set.set.strength {
                    log.log("Took no damage because defense is greater");
                    continue;
                }
                let target_name = names.get(action.target).unwrap();
                let mut damage = stats_set.set.strength - target_stats.defense;

                // collect all attack bonuses
                for (bonus, _) in (&attack_bonus, &equipped)
                    .join()
                    .filter(|(_, equip)| equip.on == attacker)
                {
                    damage = if bonus.0 >= 0 {
                        damage + bonus.0 as usize
                    } else {
                        damage.saturating_sub(bonus.0.abs() as usize)
                    };
                }

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

pub struct HealActionHandler;

impl<'a> System<'a> for HealActionHandler {
    type SystemData = (WriteStorage<'a, HealAction>, WriteStorage<'a, HealthStats>);

    fn run(&mut self, (mut heal_actions, mut healths): Self::SystemData) {
        for (healing, health) in (&heal_actions, &mut healths).join() {
            health.add_health(healing.amount);
        }

        heal_actions.clear();
    }
}


