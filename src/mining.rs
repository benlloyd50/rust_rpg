use specs::{ReadStorage, System, Join, WriteStorage, Entity};
use crate::components::{Strength, BreakAction, Breakable, SufferDamage};


pub struct MiningSystem;

impl<'a> System<'a> for MiningSystem {
    type SystemData = (
        ReadStorage<'a, Strength>,
        WriteStorage<'a, BreakAction>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Breakable>,
    );

    fn run(&mut self, (strength, mut break_actions, mut suffer_damage, breakable): Self::SystemData) {
        for (strength, action) in (&strength, &break_actions).join() {
            let target_stats = breakable.get(action.target).unwrap();
            if target_stats.defense > strength.amt {
                println!("Took no damage because defense is greater");
                continue;
            }

            let damage = strength.amt - target_stats.defense;
            println!("Dealt {} damage to {}", damage, action.target.id());
            SufferDamage::new_damage(&mut suffer_damage, action.target, -(damage as i32));
        }

        break_actions.clear()
    }
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount : vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (WriteStorage<'a, SufferDamage>,
                       WriteStorage<'a, Breakable>);

    fn run(&mut self, (mut damage, mut breakable): Self::SystemData) {
        for (mut stats, damage) in (&mut breakable, &mut damage).join() {
            let old_hp = stats.hp;
            let damage_dealt = damage.amount.iter().sum::<i32>();

            // Addition is used because damage dealt can be positive or negative
            let new_hp = stats.hp as i32 + damage_dealt;
            stats.hp = if new_hp >= 0 { new_hp as u32 } else { 0 };

            println!("Old HP: {} | Damage Dealt: {} | New HP: {}", old_hp, damage_dealt, stats.hp);
        }

        damage.clear();
    }
}
