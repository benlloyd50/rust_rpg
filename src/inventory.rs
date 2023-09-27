use bracket_terminal::prelude::{BTerm, VirtualKeyCode as VKC};
use specs::{Entity, World, WorldExt};

use crate::{
    game_init::PlayerEntity,
    player::PlayerResponse,
    AppState, State, components::{Backpack, SelectedInventoryIdx}, message_log::MessageLog,
};

pub enum UseMenuResult {
    Craft,
    _Use,
    _Drop,
    _Examine,
}

pub fn manage_player_inventory(state: &mut State, ctx: &BTerm) -> PlayerResponse {
    let player_entity: Entity;
    {
        // dirty borrow checker hack to take the value of player entity
        player_entity = state.ecs.read_resource::<PlayerEntity>().0;
    }
    match ctx.key {
        None => PlayerResponse::Waiting,
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
                _ => PlayerResponse::Waiting, // Unbound keypress so just ignore it
            }
        }
    }
}

pub fn select_item(player_entity: &Entity, idx_selected: usize, ecs: &mut World) -> PlayerResponse {
    let mut log = ecs.write_resource::<MessageLog>();
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
    let mut backpacks = ecs.write_storage::<Backpack>();
    let crafter_bag = match backpacks.get_mut(*player_entity) {
        Some(bag) => bag,
        None => panic!("Player does not have a backpack component.")
    };

    if idx_selected + 1 > crafter_bag.len() {
        log.log("Index selected is out of bounds of the backapack.");
        return PlayerResponse::Waiting;
    }
    PlayerResponse::Waiting



    // match selected_idxs.insert(
    //     *player_entity,
    //     SelectedInventoryIdx {
    //         first_idx: idx_selected,
    //         intended_action: None,
    //     },
    // ) {
    //     Ok(maybe_exisiting) => {
    //         match maybe_exisiting {
    //             Some(selection) => {
    //                 println!(
    //                     "First Index was {} and Second Index is {}",
    //                     selection.first_idx, idx_selected
    //                 );
    //
    //                 let first_item = crafter_bag.get_id_by_idx(selection.first_idx).unwrap();
    //                 let second_item = crafter_bag.get_id_by_idx(idx_selected).unwrap();
    //                 craft_item(crafter_bag, (first_item, second_item));
    //                
    //                 // Selecting process finished here clear them.
    //                 selected_idxs.remove(*player_entity);
    //                 PlayerResponse::Waiting
    //             }
    //             None => {
    //                 println!("Inserted first selected inventory index which was {}", idx_selected);
    //                 PlayerResponse::Waiting
    //             }
    //         }
    //     }
    //     Err(_) => {
    //         eprintln!("Cannot access player's selected index and failed during insertion into SelectedInventoryIdx storage");
    //         PlayerResponse::Waiting
    //     }
    // }
}

fn clean_and_exit_inventory(player_entity: &Entity, ecs: &mut World) -> PlayerResponse {
    // TODO: remove temporary inventory related components
    let mut selected_idxs = ecs.write_storage::<SelectedInventoryIdx>();
    selected_idxs.remove(*player_entity);
    PlayerResponse::StateChange(AppState::InGame)
}
