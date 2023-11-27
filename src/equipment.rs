// Equipment
// ui: select item -> 'equip' ->
//      we know the idx selected (item) and that the player (since it's ui) is the equipper,
// monster ai: ai decision making -> 'equip' -> should know the item entity since it would be in_bag
//
// system: toggle_equip(item entity) -> check item is equipable && equipper can equip (slot in equipper's slots)

use std::mem::discriminant;

use specs::{Entities, Join, ReadStorage, System, WriteStorage};

use crate::components::{EquipAction, Equipable, EquipmentSlots, Equipped};

pub struct EquipActionHandler;

impl<'a> System<'a> for EquipActionHandler {
    type SystemData = (
        WriteStorage<'a, EquipAction>,
        WriteStorage<'a, Equipped>,
        ReadStorage<'a, EquipmentSlots>,
        ReadStorage<'a, Equipable>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut equip_actions, mut equippeds, equipment_slots, equipables, entities): Self::SystemData,
    ) {
        for (equipper, equip, equip_slots) in (&entities, &equip_actions, &equipment_slots).join() {
            let mut can_equip = true;
            match equipables.get(equip.item) {
                Some(target_equipable) => {
                    let target_slots_available = equip_slots
                        .slots
                        .iter()
                        .filter(|s| discriminant(*s) == discriminant(&target_equipable.slot))
                        .count();
                    let equipped_in_target_slot = (&equippeds, &equipables)
                        .join()
                        .filter(|(equipped, equipable)| {
                            equipped.on == equipper && equipable.slot == target_equipable.slot
                        })
                        .count();
                    if equipped_in_target_slot >= target_slots_available {
                        can_equip = false;
                    }
                }
                None => {
                    continue;
                } // item trying to be equipped is not equipable
            }

            match equippeds.get(equip.item) {
                Some(_) => {
                    // if the item is, then just remove
                    equippeds.remove(equip.item);
                }
                None => {
                    // else add equipped to the item
                    if !can_equip {
                        continue;
                    }
                    let _ = equippeds.insert(equip.item, Equipped { on: equipper });
                }
            }
        }

        equip_actions.clear();
    }
}
