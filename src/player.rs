use crate::{
    components::{BreakAction, FinishedActivity, FishAction, Name, PickupAction},
    game_init::PlayerEntity,
    items::{inventory_contains, try_item},
    map::{Map, TileEntity},
    message_log::MessageLog,
    AppState, Position, State,
};
use bracket_terminal::prelude::{BTerm, Point, VirtualKeyCode as VKC};
use specs::{prelude::*, Component};
use std::process::exit;

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Player;

pub enum PlayerResponse {
    StateChange(AppState),
    TurnAdvance,
    Waiting,
}

pub fn manage_player_input(ecs: &mut World, ctx: &BTerm) -> PlayerResponse {
    match ctx.key {
        None => PlayerResponse::Waiting,
        Some(key) => {
            match key {
                VKC::W | VKC::Up => try_move_player(0, -1, ecs),
                VKC::S | VKC::Down => try_move_player(0, 1, ecs),
                VKC::A | VKC::Left => try_move_player(-1, 0, ecs),
                VKC::D | VKC::Right => try_move_player(1, 0, ecs),
                VKC::P => try_pickup(ecs), // p for pickup
                VKC::I => PlayerResponse::StateChange(AppState::PlayerInInventory),
                VKC::Back => exit(0),
                VKC::Space => {
                    let mut log = ecs.fetch_mut::<MessageLog>();
                    log.log("The player stands around.");
                    PlayerResponse::TurnAdvance
                }
                _ => PlayerResponse::Waiting, // Unbound keypress so just ignore it
            }
        }
    }
}

// ui x = 81 for shift arrow
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
                VKC::Key1 => try_item(&player_entity, 1, &mut state.ecs),
                VKC::Key2 => try_item(&player_entity, 2, &mut state.ecs),
                VKC::Key3 => try_item(&player_entity, 3, &mut state.ecs),
                VKC::Key4 => try_item(&player_entity, 4, &mut state.ecs),
                VKC::Key5 => try_item(&player_entity, 5, &mut state.ecs),
                VKC::Key6 => try_item(&player_entity, 6, &mut state.ecs),
                VKC::Key7 => try_item(&player_entity, 7, &mut state.ecs),
                VKC::Key8 => try_item(&player_entity, 8, &mut state.ecs),
                VKC::Key9 => try_item(&player_entity, 9, &mut state.ecs),
                VKC::Key0 => try_item(&player_entity, 10, &mut state.ecs),
                VKC::Minus => try_item(&player_entity, 11, &mut state.ecs),
                VKC::Plus => try_item(&player_entity, 12, &mut state.ecs),
                VKC::A => try_item(&player_entity, 13, &mut state.ecs),
                VKC::B => try_item(&player_entity, 14, &mut state.ecs),
                VKC::C => try_item(&player_entity, 15, &mut state.ecs),
                VKC::D => try_item(&player_entity, 16, &mut state.ecs),
                VKC::E => try_item(&player_entity, 17, &mut state.ecs),
                VKC::F => try_item(&player_entity, 18, &mut state.ecs),
                VKC::G => try_item(&player_entity, 19, &mut state.ecs),
                VKC::H => try_item(&player_entity, 20, &mut state.ecs),
                VKC::Escape | VKC::I => PlayerResponse::StateChange(AppState::InGame),
                _ => PlayerResponse::Waiting, // Unbound keypress so just ignore it
            }
        }
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> PlayerResponse {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut break_actions = ecs.write_storage::<BreakAction>();
    let mut fish_actions = ecs.write_storage::<FishAction>();

    for (player_entity, pos, _) in (&entities, &mut positions, &players).join() {
        let target_pos = Point::new(pos.x as i32 + delta_x, pos.y as i32 + delta_y);

        // check target_pos is in map bounds
        if target_pos.x < 0
            || target_pos.y < 0
            || target_pos.x >= map.width as i32
            || target_pos.y >= map.height as i32
        {
            return PlayerResponse::Waiting;
        }

        match map.first_entity_in_pos(&Position::from(target_pos)) {
            Some(tile) => match tile {
                TileEntity::Blocking => {
                    println!("Map is blocked at {}, {}", target_pos.x, target_pos.y);
                    return PlayerResponse::Waiting;
                }
                TileEntity::Fishable(_entity) => {
                    println!("Attempting to fish at {}, {}", target_pos.x, target_pos.y);
                    if inventory_contains(&Name::new("Fishing Rod"), &player_entity, ecs) {
                        fish_actions
                            .insert(
                                player_entity,
                                FishAction {
                                    target: target_pos.into(),
                                },
                            )
                            .expect("Fish action could not be added to player entity");
                        return PlayerResponse::StateChange(AppState::activity_bound());
                    }
                }
                TileEntity::Breakable(entity) => {
                    println!(
                        "Map is breakable at {}, {} : id: {}",
                        target_pos.x,
                        target_pos.y,
                        entity.id()
                    );
                    break_actions
                        .insert(player_entity, BreakAction { target: *entity })
                        .expect("Break action could not be added to player entity");
                    return PlayerResponse::TurnAdvance;
                }
                TileEntity::Item(_) => {
                    pos.x = target_pos.x as usize;
                    pos.y = target_pos.y as usize;
                    return PlayerResponse::TurnAdvance;
                }
            },
            None => {
                pos.x = target_pos.x as usize;
                pos.y = target_pos.y as usize;
                return PlayerResponse::TurnAdvance;
            }
        }
    }

    PlayerResponse::Waiting
}

fn try_pickup(ecs: &mut World) -> PlayerResponse {
    let mut pickups = ecs.write_storage::<PickupAction>();

    let player_entity = ecs.read_resource::<PlayerEntity>();
    let positions = ecs.read_storage::<Position>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    if let Some((player_entity, pos)) = (&entities, &positions)
        .join()
        .find(|(e, _)| e.eq(&player_entity.0))
    {
        let mut item_iter = map.all_items_at_pos(&pos);
        if let Some(item_entity) = item_iter.next() {
            let _ = pickups.insert(
                player_entity,
                PickupAction {
                    item: *item_entity.as_item_entity().unwrap(),
                },
            );
        }
    }

    PlayerResponse::Waiting
}

/// Checks if the main player entity has a FinishedActivity component on it so we can return to
/// InGame state. Will not work nicely if we have multiple player entities, which we shouldn't ever
pub fn check_player_activity(ecs: &mut World) -> bool {
    let players = ecs.read_storage::<Player>();
    let mut finished_activities = ecs.write_storage::<FinishedActivity>();

    for _ in (&players, &mut finished_activities).join() {
        return true;
    }

    false
}
