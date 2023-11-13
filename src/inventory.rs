use bracket_terminal::prelude::{BTerm, VirtualKeyCode as VKC};
use specs::{Entity, Join, World, WorldExt};

use crate::{
    components::{InBag, Item, Name, SelectedInventoryIdx, WantsToCraft, WantsToEquip},
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
    Cancel,
}

pub enum InventoryResponse {
    Waiting,
    ActionReady,
    SecondItemSelected { second_idx: usize },
    StateChange(AppState),
}

pub fn handle_player_input(ecs: &mut World, ctx: &BTerm) -> InventoryResponse {
    let player_entity: Entity;
    {
        // dirty borrow checker hack to take the value of player entity
        player_entity = ecs.read_resource::<PlayerEntity>().0;
    }
    match ctx.key {
        None => InventoryResponse::Waiting,
        Some(key) if check_inventory_selection(&ecs) == SelectionStatus::SelectionWithoutAction => {
            let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
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
                VKC::Key1 => select_item(&player_entity, 0, ecs),
                VKC::Key2 => select_item(&player_entity, 1, ecs),
                VKC::Key3 => select_item(&player_entity, 2, ecs),
                VKC::Key4 => select_item(&player_entity, 3, ecs),
                VKC::Key5 => select_item(&player_entity, 4, ecs),
                VKC::Key6 => select_item(&player_entity, 5, ecs),
                VKC::Key7 => select_item(&player_entity, 6, ecs),
                VKC::Key8 => select_item(&player_entity, 7, ecs),
                VKC::Key9 => select_item(&player_entity, 8, ecs),
                VKC::Key0 => select_item(&player_entity, 9, ecs),
                VKC::Minus => select_item(&player_entity, 10, ecs),
                VKC::Plus => select_item(&player_entity, 11, ecs),
                VKC::A => select_item(&player_entity, 12, ecs),
                VKC::B => select_item(&player_entity, 13, ecs),
                VKC::C => select_item(&player_entity, 14, ecs),
                VKC::D => select_item(&player_entity, 15, ecs),
                VKC::E => select_item(&player_entity, 16, ecs),
                VKC::F => select_item(&player_entity, 17, ecs),
                VKC::G => select_item(&player_entity, 18, ecs),
                VKC::H => select_item(&player_entity, 19, ecs),
                VKC::Escape | VKC::I => clean_and_exit_inventory(&player_entity, ecs),
                _ => InventoryResponse::Waiting, // Unbound keypress so just ignore it
            }
        }
    }
}

fn select_item(player_entity: &Entity, idx_selected: usize, ecs: &mut World) -> InventoryResponse {
    let mut log = ecs.write_resource::<MessageLog>();
    let in_bags = ecs.read_storage::<InBag>();
    // why do i gotta join 1 storage? is there a better way?
    let inv_count = (&in_bags)
        .join()
        .filter(|bag| bag.owner == *player_entity)
        .count();

    if idx_selected + 1 > inv_count {
        log.log("Index selected is out of bounds of the backapack.");
        return InventoryResponse::Waiting;
    }

    match check_inventory_selection(ecs) {
        SelectionStatus::NoSelection => {
            let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
            let _ = selected_idxs.insert(
                *player_entity,
                SelectedInventoryIdx {
                    first_idx: idx_selected,
                    intended_action: None,
                },
            );
            InventoryResponse::Waiting
        }
        SelectionStatus::SelectionWithoutAction => InventoryResponse::Waiting,
        SelectionStatus::SelectionAndAction => {
            // The first idx will be on the component
            InventoryResponse::SecondItemSelected {
                second_idx: idx_selected,
            }
        }
    }
}

pub fn handle_one_item_actions(ecs: &mut World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
    let selection = match selected_idxs.get(player_entity.0) {
        Some(idx) => idx,
        None => {
            eprintln!(
                "Player has no SelectedInventoryIdx component associated when using one item"
            );
            return;
        }
    };

    let mut log = ecs.write_resource::<MessageLog>();
    match selection.intended_action.as_ref().unwrap() {
        UseMenuResult::Drop => {
            // remove item from bag
            log.log("Dropped it");
        }
        UseMenuResult::Examine => {
            //log flavor text
            let items = ecs.read_storage::<Item>();
            let in_bags = ecs.read_storage::<InBag>();
            let names = ecs.read_storage::<Name>();
            if let Some((idx, (_, _, Name(name)))) = (&items, &in_bags, &names)
                .join()
                .enumerate()
                .filter(|(idx, (_, bag, _))| {
                    idx == &selection.first_idx && bag.owner == player_entity.0
                })
                .next()
            {
                log.log(format!("Examined the {} at {}", name, idx + 1));
            } else {
                log.log(format!("Couldn't examine idx: {}", selection.first_idx));
            }
        }
        UseMenuResult::Equip => {
            // get the item at the selected idx and create action for it
            let items = ecs.read_storage::<Item>();
            let in_bags = ecs.read_storage::<InBag>();
            let entities = ecs.entities();
            if let Some((item_entity, _, _)) = (&entities, &items, &in_bags)
                .join()
                .filter(|(_, _, bag)| bag.owner == player_entity.0)
                .nth(selection.first_idx)
            {
                let mut equip_actions = ecs.write_storage::<WantsToEquip>();
                let _ = equip_actions.insert(player_entity.0, WantsToEquip { item: item_entity });
            }
        }
        UseMenuResult::Craft => {
            unreachable!("Two item actions cannot be performed here (in this fn).")
        }
        UseMenuResult::Cancel => {}
    }

    selected_idxs.remove(player_entity.0);
}

pub fn handle_two_item_actions(ecs: &mut World, second_idx: usize) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
    let selection = match selected_idxs.get(player_entity.0) {
        Some(idx) => idx.clone(),
        None => {
            eprintln!(
                "Player has no SelectedInventoryIdx component associated when using two items"
            );
            return;
        }
    };

    // unwrap is safe because this fn is not entered unless we have an action and selection selected
    match selection.intended_action.as_ref().unwrap() {
        UseMenuResult::Craft => {
            if selection.first_idx == second_idx {
                eprintln!("Cannot craft using the same item in your inventory.");
                return;
            }
            let mut wants_to_craft = ecs.write_storage::<WantsToCraft>();
            let _ = wants_to_craft.insert(
                player_entity.0,
                WantsToCraft {
                    first_idx: selection.first_idx,
                    second_idx,
                },
            );
        }
        _ => unreachable!(
            "These options should be unreachable since they only require 1 item to be performed."
        ),
    }

    selected_idxs.remove(player_entity.0);
}

fn clean_and_exit_inventory(player_entity: &Entity, ecs: &mut World) -> InventoryResponse {
    // TODO: remove temporary inventory related components
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
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
    let selected_idxs = ecs.read_storage::<SelectedInventoryIdx>();
    match selected_idxs.get(player_entity.0) {
        Some(selection) => match &selection.intended_action {
            Some(_unperformed_action) => SelectionStatus::SelectionAndAction,
            None => SelectionStatus::SelectionWithoutAction,
        },
        None => SelectionStatus::NoSelection,
    }
}
