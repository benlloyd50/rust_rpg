use crate::{
    components::{
        AttackAction, BreakAction, FinishedActivity, FishAction, Interactor, InteractorMode, Name,
        PickupAction,
    },
    game_init::PlayerEntity,
    items::inventory_contains,
    map::{Map, TileEntity},
    ui::message_log::MessageLog,
    AppState, Position,
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
                VKC::M => {
                    switch_interaction_mode(ecs);
                    PlayerResponse::Waiting
                }
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

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> PlayerResponse {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let interactors = ecs.read_storage::<Interactor>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut break_actions = ecs.write_storage::<BreakAction>();
    let mut fish_actions = ecs.write_storage::<FishAction>();
    let mut attack_actions = ecs.write_storage::<AttackAction>();

    for (player_entity, pos, interactor, _) in
        (&entities, &mut positions, &interactors, &players).join()
    {
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
                TileEntity::Blocking(blocker) => match interactor.mode {
                    InteractorMode::Reactive => {
                        return PlayerResponse::Waiting;
                    }
                    InteractorMode::Agressive => {
                        attack_actions
                            .insert(player_entity, AttackAction { target: *blocker })
                            .expect("Attack action could not be added to player entity");
                        return PlayerResponse::TurnAdvance;
                    }
                },
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

fn switch_interaction_mode(ecs: &mut World) {
    let player_entity = ecs.read_resource::<PlayerEntity>();
    let mut interactors = ecs.write_storage::<Interactor>();

    let player_interact = match interactors.get_mut(player_entity.0) {
        Some(p) => p,
        None => {
            eprintln!("Player has no interactor component...");
            return;
        }
    };

    player_interact.mode = match player_interact.mode {
        InteractorMode::Reactive => InteractorMode::Agressive,
        InteractorMode::Agressive => InteractorMode::Reactive,
    };
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
