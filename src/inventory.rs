use bracket_terminal::prelude::{BTerm, VirtualKeyCode as VKC};
use specs::{Entity, World, WorldExt};

use crate::{
    components::{Backpack, SelectedInventoryIdx},
    crafting::craft_item,
    game_init::PlayerEntity,
    message_log::MessageLog,
    AppState, State,
};

pub enum UseMenuResult {
    Craft,
    Drop,
    Examine,
    Cancel,
}

pub enum InventoryResponse {
    Waiting,
    ActionReady,
    SecondItemSelected { second_idx: usize },
    StateChange(AppState),
}

pub fn inventory_tick(new_state: &mut AppState, state: &mut State, ctx: &BTerm) {
    match handle_player_input(state, ctx) {
        InventoryResponse::Waiting => {
            // Player hasn't done anything yet so only run essential systems
        }
        InventoryResponse::ActionReady => {
            handle_one_item_actions(&mut state.ecs);
        }
        InventoryResponse::SecondItemSelected { second_idx } => {
            handle_two_item_actions(&mut state.ecs, second_idx);
        }
        InventoryResponse::StateChange(delta_state) => {
            *new_state = delta_state;
        }
    }
}

fn handle_player_input(state: &mut State, ctx: &BTerm) -> InventoryResponse {
    let player_entity: Entity;
    {
        // dirty borrow checker hack to take the value of player entity
        player_entity = state.ecs.read_resource::<PlayerEntity>().0;
    }
    match ctx.key {
        None => InventoryResponse::Waiting,
        Some(key)
            if check_player_selection(&state.ecs) == SelectionStatus::SelectionWithoutAction =>
        {
            let mut selected_idxs = state.ecs.write_storage::<SelectedInventoryIdx>();
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
                VKC::Key1 => select_item(&player_entity, 0, &mut state.ecs),
                VKC::Key2 => select_item(&player_entity, 1, &mut state.ecs),
                VKC::Key3 => select_item(&player_entity, 2, &mut state.ecs),
                VKC::Key4 => select_item(&player_entity, 3, &mut state.ecs),
                VKC::Key5 => select_item(&player_entity, 4, &mut state.ecs),
                VKC::Key6 => select_item(&player_entity, 5, &mut state.ecs),
                VKC::Key7 => select_item(&player_entity, 6, &mut state.ecs),
                VKC::Key8 => select_item(&player_entity, 7, &mut state.ecs),
                VKC::Key9 => select_item(&player_entity, 8, &mut state.ecs),
                VKC::Key0 => select_item(&player_entity, 9, &mut state.ecs),
                VKC::Minus => select_item(&player_entity, 10, &mut state.ecs),
                VKC::Plus => select_item(&player_entity, 11, &mut state.ecs),
                VKC::A => select_item(&player_entity, 12, &mut state.ecs),
                VKC::B => select_item(&player_entity, 13, &mut state.ecs),
                VKC::C => select_item(&player_entity, 14, &mut state.ecs),
                VKC::D => select_item(&player_entity, 15, &mut state.ecs),
                VKC::E => select_item(&player_entity, 16, &mut state.ecs),
                VKC::F => select_item(&player_entity, 17, &mut state.ecs),
                VKC::G => select_item(&player_entity, 18, &mut state.ecs),
                VKC::H => select_item(&player_entity, 19, &mut state.ecs),
                VKC::Escape | VKC::I => clean_and_exit_inventory(&player_entity, &mut state.ecs),
                _ => InventoryResponse::Waiting, // Unbound keypress so just ignore it
            }
        }
    }
}

fn select_item(player_entity: &Entity, idx_selected: usize, ecs: &mut World) -> InventoryResponse {
    let mut log = ecs.write_resource::<MessageLog>();
    let mut backpacks = ecs.write_storage::<Backpack>();
    let crafter_bag = match backpacks.get_mut(*player_entity) {
        Some(bag) => bag,
        None => panic!("Player does not have a backpack component."),
    };

    if idx_selected + 1 > crafter_bag.len() {
        log.log("Index selected is out of bounds of the backapack.");
        return InventoryResponse::Waiting;
    }

    match check_player_selection(ecs) {
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

fn handle_one_item_actions(ecs: &mut World) {
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
            println!("pretend you dropped it");
        }
        UseMenuResult::Examine => {
            //log flavor text
            log.log("Examined it");
        }
        UseMenuResult::Cancel => {}
        UseMenuResult::Craft => {
            unreachable!("Two item actions cannot be performed here (in this fn).")
        }
    }

    selected_idxs.remove(player_entity.0);
}

fn handle_two_item_actions(ecs: &mut World, second_idx: usize) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut backpacks = ecs.write_storage::<Backpack>();
    let player_bag = match backpacks.get_mut(player_entity.0) {
        Some(bag) => bag,
        None => {
            eprintln!("Player has no backpack component associated with them.");
            return;
        }
    };

    let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
    let selection = match selected_idxs.get(player_entity.0) {
        Some(idx) => idx,
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
            perform_craft(player_bag, selection.first_idx, second_idx);
        }
        _ => unreachable!(
            "These options should be unreachable since they only require 1 item to be performed."
        ),
    }

    selected_idxs.remove(player_entity.0);
}

fn perform_craft(crafter_bag: &mut Backpack, first_idx: usize, second_idx: usize) {
    if first_idx == second_idx {
        eprintln!("Cannot craft using the same item in your inventory.")
    }
    let first_item = crafter_bag.get_id_by_idx(first_idx).unwrap();
    let second_item = crafter_bag.get_id_by_idx(second_idx).unwrap();
    craft_item(crafter_bag, first_item, second_item);
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
pub fn check_player_selection(ecs: &World) -> SelectionStatus {
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
