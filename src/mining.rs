use specs::{ReadStorage, System, Join, WriteStorage, Component, Entity, VecStorage};
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
            let target_defense = breakable.get(action.target).unwrap();
            if target_defense.hp < strength.amt {
                let damage = strength.amt - target_defense.hp;
                SufferDamage::new_damage(&mut suffer_damage, action.target, damage);
            }
        }

        break_actions.clear()
    }
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: u32) {
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
            stats.hp -= damage.amount.iter().sum::<u32>();
        }

        damage.clear();
    }
}
