use crate::{
    components::{BreakAction, FinishedActivity, FishAction, Name, PickupAction},
    items::inventory_contains,
    map::{Map, TileEntity},
    AppState, Position, State,
};
use bracket_terminal::prelude::{BTerm, Point, VirtualKeyCode};
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

pub fn manage_player_input(state: &mut State, ctx: &BTerm) -> PlayerResponse {
    match ctx.key {
        None => PlayerResponse::Waiting,
        Some(key) => {
            match key {
                VirtualKeyCode::W | VirtualKeyCode::Up => try_move_player(0, -1, &mut state.ecs),
                VirtualKeyCode::S | VirtualKeyCode::Down => try_move_player(0, 1, &mut state.ecs),
                VirtualKeyCode::A | VirtualKeyCode::Left => try_move_player(-1, 0, &mut state.ecs),
                VirtualKeyCode::D | VirtualKeyCode::Right => try_move_player(1, 0, &mut state.ecs),
                VirtualKeyCode::P => try_pickup(&mut state.ecs), // p for pickup
                VirtualKeyCode::Escape => exit(0),
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
                    if let Some(_item) =
                        inventory_contains(&Name::new("Fishing Rod"), &player_entity, ecs)
                    {
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

    let positions = ecs.read_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    for (player_entity, pos, _) in (&entities, &positions, &players).join() {
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
