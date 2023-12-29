use bracket_lib::terminal::{BTerm, VirtualKeyCode as VKC};
use itertools::Itertools;
use specs::{Entity, Join, World, WorldExt};
use specs::{LendJoin, ReadStorage};

use crate::components::{ConsumeAction, Equipped, Position};
use crate::config::{InventoryConfig, SortMode};
use crate::data_read::ENTITY_DB;
use crate::{
    components::{CraftAction, EquipAction, InBag, Item, Name, SelectedInventoryItem},
    game_init::PlayerEntity,
    ui::message_log::MessageLog,
    AppState,
};

#[derive(Clone)]
pub enum UseMenuResult {
    Craft,
    Drop,
    Examine,
    Equip,
    Consume,
    Cancel,
}

pub enum InventoryResponse {
    Waiting,
    ActionReady,
    SecondItemSelected { second_item: Entity },
    StateChange(AppState),
}

pub fn p_input_inventory(ecs: &mut World, ctx: &BTerm, cfg: &mut InventoryConfig) -> InventoryResponse {
    let player_entity: Entity;
    {
        // dirty borrow checker hack to take the value of player entity
        player_entity = ecs.read_resource::<PlayerEntity>().0;
    }
    match ctx.key {
        None => InventoryResponse::Waiting,
        Some(key) if check_inventory_selection(ecs) == SelectionStatus::SelectionWithoutAction => {
            let mut selected_idxs = ecs.write_storage::<SelectedInventoryItem>();
            if let Some(selection) = selected_idxs.get_mut(player_entity) {
                match key {
                    VKC::U => {
                        // using an item with something else translates to crafting most of the time (99.9%)
                        selection.intended_action = Some(UseMenuResult::Craft);
                        InventoryResponse::Waiting
                    }
                    VKC::E => {
                        selection.intended_action = Some(UseMenuResult::Examine);
                        InventoryResponse::ActionReady
                    }
                    VKC::D => {
                        selection.intended_action = Some(UseMenuResult::Drop);
                        InventoryResponse::ActionReady
                    }
                    VKC::Q => {
                        selection.intended_action = Some(UseMenuResult::Equip);
                        InventoryResponse::ActionReady
                    }
                    VKC::C => {
                        selection.intended_action = Some(UseMenuResult::Consume);
                        InventoryResponse::ActionReady
                    }
                    VKC::Escape => {
                        selection.intended_action = Some(UseMenuResult::Cancel);
                        InventoryResponse::ActionReady
                    }
                    _ => InventoryResponse::Waiting,
                }
            } else {
                InventoryResponse::Waiting
            }
        }
        Some(key) => {
            match key {
                VKC::Key1 => select_item(&player_entity, 0, ecs, cfg),
                VKC::Key2 => select_item(&player_entity, 1, ecs, cfg),
                VKC::Key3 => select_item(&player_entity, 2, ecs, cfg),
                VKC::Key4 => select_item(&player_entity, 3, ecs, cfg),
                VKC::Key5 => select_item(&player_entity, 4, ecs, cfg),
                VKC::Key6 => select_item(&player_entity, 5, ecs, cfg),
                VKC::Key7 => select_item(&player_entity, 6, ecs, cfg),
                VKC::Key8 => select_item(&player_entity, 7, ecs, cfg),
                VKC::Key9 => select_item(&player_entity, 8, ecs, cfg),
                VKC::A => select_item(&player_entity, 9, ecs, cfg),
                VKC::B => select_item(&player_entity, 10, ecs, cfg),
                VKC::C => select_item(&player_entity, 11, ecs, cfg),
                VKC::D => select_item(&player_entity, 12, ecs, cfg),
                VKC::E => select_item(&player_entity, 13, ecs, cfg),
                VKC::F => select_item(&player_entity, 14, ecs, cfg),
                VKC::G => select_item(&player_entity, 15, ecs, cfg),
                VKC::H => select_item(&player_entity, 16, ecs, cfg),
                VKC::S => {
                    cfg.rotate_sort_mode();
                    InventoryResponse::Waiting
                }
                VKC::Escape | VKC::I => clean_and_exit_inventory(&player_entity, ecs),
                _ => InventoryResponse::Waiting, // Unbound keypress so just ignore it
            }
        }
    }
}

// This function depends on the ui code of getting the iterator
fn select_item(
    player_entity: &Entity,
    idx_selected: usize,
    ecs: &mut World,
    cfg: &InventoryConfig,
) -> InventoryResponse {
    let items: ReadStorage<Item> = ecs.read_storage();
    let inbags: ReadStorage<InBag> = ecs.read_storage();
    let names: ReadStorage<Name> = ecs.read_storage();
    let equipped: ReadStorage<Equipped> = ecs.read_storage();

    let entities = ecs.entities();
    let selected_entity = (&entities, &items, &inbags, &names, (&equipped).maybe())
        // important: this must match in src/ui/inventory.rs until a better solution is found to share code
        // up to the sorted_by
        .join()
        .filter(|(_, _, bag, _, _)| bag.owner == *player_entity)
        .sorted_by(|a, b| match cfg.sort_mode {
            SortMode::NameABC => a.3.cmp(b.3),
            SortMode::IDAsc => a.1.id.cmp(&b.1.id),
            _ => a.1.id.cmp(&b.1.id),
        })
        .nth(idx_selected)
        .map(|(e, _, _, _, _)| e);

    match check_inventory_selection(ecs) {
        SelectionStatus::NoSelection => {
            let mut selected_idxs = ecs.write_storage::<SelectedInventoryItem>();
            if let Some(first_item) = selected_entity {
                let _ =
                    selected_idxs.insert(*player_entity, SelectedInventoryItem { first_item, intended_action: None });
            } else {
                let mut log = ecs.write_resource::<MessageLog>();
                log.log("Index selected is out of bounds of the backapack.");
            }
            // good to insert because we know the player does not have one currently
            InventoryResponse::Waiting
        }
        SelectionStatus::SelectionWithoutAction => InventoryResponse::Waiting,
        SelectionStatus::SelectionAndAction => {
            // The first idx will be on the component
            if let Some(second_item) = selected_entity {
                InventoryResponse::SecondItemSelected { second_item }
            } else {
                InventoryResponse::Waiting
            }
        }
    }
}

pub fn handle_one_item_actions(ecs: &mut World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryItem>();
    let selection = match selected_idxs.get(player_entity.0) {
        Some(item) => item,
        None => {
            eprintln!("Player has no SelectedInventoryIdx component associated when using one item");
            return;
        }
    };
    let mut items = ecs.write_storage::<Item>();
    let mut in_bags = ecs.write_storage::<InBag>();
    let entities = ecs.entities();
    let items_in_player_bag = (&entities, &items, &in_bags)
        .join()
        .find(|(item_entity, _, bag)| item_entity == &selection.first_item && bag.owner == player_entity.0);

    let mut log = ecs.write_resource::<MessageLog>();
    match selection.intended_action.as_ref().unwrap() {
        UseMenuResult::Drop => {
            // remove item from bag
            if let Some((item_entity, dropped_item, _)) = items_in_player_bag {
                in_bags.remove(item_entity);
                log.log("Dropped it");

                let mut positions = ecs.write_storage::<Position>();
                if let Some(player_position) = positions.get(player_entity.0) {
                    if let Some(ground_item) = (&entities, &items, &positions)
                        .join()
                        .find(|(_, i, p)| p.eq(&player_position) && i.id.eq(&dropped_item.id))
                    {
                        let _ = items
                            .insert(ground_item.0, Item::new(ground_item.1.id, ground_item.1.qty + dropped_item.qty));
                    } else {
                        let _ = positions.insert(item_entity, *player_position);
                    }
                }
            }
        }
        UseMenuResult::Examine => {
            //log flavor text
            if let Some((_, item, _)) = items_in_player_bag {
                let examine_text = match &ENTITY_DB.lock().unwrap().items.get_by_id(item.id) {
                    Some(info) => info.examine_text.clone(),
                    None => format!("Could not find item with id: {}", item.id),
                };
                log.log(examine_text);
            } else {
                log.log(format!("Couldn't examine entity: {:?}", selection.first_item));
            }
        }
        UseMenuResult::Equip => {
            // get the item at the selected idx and create action for it
            if let Some((item_entity, _, _)) = items_in_player_bag {
                let mut equip_actions = ecs.write_storage::<EquipAction>();
                let _ = equip_actions.insert(player_entity.0, EquipAction { item: item_entity });
            }
        }
        UseMenuResult::Consume => {
            if let Some((item_entity, _, _)) = items_in_player_bag {
                let mut equip_actions = ecs.write_storage::<ConsumeAction>();
                let _ = equip_actions.insert(player_entity.0, ConsumeAction::new(&item_entity));
            }
        }
        UseMenuResult::Craft => {
            unreachable!("Two item actions cannot be performed here (in this fn).")
        }
        UseMenuResult::Cancel => {}
    }

    selected_idxs.remove(player_entity.0);
}

pub fn handle_two_item_actions(ecs: &mut World, second_item: &Entity) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryItem>();
    let selection = match selected_idxs.get(player_entity.0) {
        Some(idx) => idx.clone(),
        None => {
            eprintln!("Player has no SelectedInventoryIdx component associated when using two items");
            return;
        }
    };

    // unwrap is safe because this fn is not entered unless we have an action and selection selected
    match selection.intended_action.as_ref().unwrap() {
        UseMenuResult::Craft => {
            if selection.first_item == *second_item {
                eprintln!("Cannot craft using the same item in your inventory.");
                return;
            }
            let mut craft_actions = ecs.write_storage::<CraftAction>();
            let _ = craft_actions
                .insert(player_entity.0, CraftAction { first_item: selection.first_item, second_item: *second_item });
        }
        _ => unreachable!("These options should be unreachable since they only require 1 item to be performed."),
    }

    selected_idxs.remove(player_entity.0);
}

fn clean_and_exit_inventory(player_entity: &Entity, ecs: &mut World) -> InventoryResponse {
    // TODO: remove temporary inventory related components
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryItem>();
    selected_idxs.remove(*player_entity);
    InventoryResponse::StateChange(AppState::InGame)
}

#[derive(PartialEq, Eq)]
pub enum SelectionStatus {
    NoSelection,
    SelectionWithoutAction,
    SelectionAndAction,
}

/// Gets the state of the player in the inventory screen.
/// Checks if the player has made a selection and an action for the selection
pub fn check_inventory_selection(ecs: &World) -> SelectionStatus {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let selected_idxs = ecs.read_storage::<SelectedInventoryItem>();
    match selected_idxs.get(player_entity.0) {
        Some(selection) => match &selection.intended_action {
            Some(_unperformed_action) => SelectionStatus::SelectionAndAction,
            None => SelectionStatus::SelectionWithoutAction,
        },
        None => SelectionStatus::NoSelection,
    }
}
