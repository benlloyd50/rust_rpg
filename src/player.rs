use crate::{
    audio::play_sound_effect,
    components::{
        AttackAction, BreakAction, FinishedActivity, FishAction, GameAction, Interactor, InteractorMode, Name,
        PickupAction,
    },
    game_init::PlayerEntity,
    items::inventory_contains,
    map::{MapRes, TileEntity},
    saveload::{save_game_exists, SaveAction},
    settings::SettingsAction,
    ui::message_log::MessageLog,
    AppState, Position,
};
use bracket_lib::terminal::{BTerm, Point, VirtualKeyCode as VKC};
use log::info;
use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};

#[derive(Default, Component, Clone, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Player {}

pub enum PlayerResponse {
    StateChange(AppState),
    TurnAdvance,
    Waiting,
}

pub fn p_input_game(ecs: &mut World, ctx: &BTerm) -> PlayerResponse {
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
                VKC::I => {
                    play_sound_effect("ui_inventory");
                    PlayerResponse::StateChange(AppState::PlayerInInventory)
                }
                VKC::Escape => PlayerResponse::StateChange(AppState::SaveGame),
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
    let entities = ecs.entities();
    for (player_entity, pos, interactor, _) in (&entities, &mut positions, &interactors, &players).join() {
        let target_pos = Point::new(pos.x as i32 + delta_x, pos.y as i32 + delta_y);

        let map = ecs.fetch::<MapRes>();
        if !map.0.in_bounds(target_pos) {
            return PlayerResponse::Waiting;
        }

        match map.0.first_entity_in_pos(&Position::from(target_pos)) {
            Some(tile) => match tile {
                TileEntity::Breakable(entity) => {
                    info!("Map is breakable at {}, {} : id: {}", target_pos.x, target_pos.y, entity.id());
                    ecs.write_storage::<BreakAction>()
                        .insert(player_entity, BreakAction { target: *entity })
                        .expect("Break action could not be added to player entity");
                    return PlayerResponse::TurnAdvance;
                }
                TileEntity::Blocking(blocker) => match interactor.mode {
                    InteractorMode::Reactive => {
                        return PlayerResponse::Waiting;
                    }
                    InteractorMode::Agressive => {
                        info!("Player attacked the position, {:?}", target_pos);
                        ecs.write_storage::<AttackAction>()
                            .insert(player_entity, AttackAction { target: *blocker })
                            .expect("Attack action could not be added to player entity");
                        return PlayerResponse::TurnAdvance;
                    }
                },
                TileEntity::Fishable(_entity) => {
                    info!("Attempting to fish at {}, {}", target_pos.x, target_pos.y);
                    if inventory_contains(&Name::new("Fishing Rod"), &player_entity, ecs) {
                        ecs.write_storage::<FishAction>()
                            .insert(player_entity, FishAction { target: target_pos.into() })
                            .expect("Fish action could not be added to player entity");
                        return PlayerResponse::StateChange(AppState::activity_bound());
                    }
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
    let map = ecs.fetch::<MapRes>();

    if let Some(pos) = positions.get(player_entity.0) {
        let mut item_iter = map.0.all_items_at_pos(pos);
        if let Some(item_entity) = item_iter.next() {
            let _ = pickups.insert(player_entity.0, PickupAction { item: *item_entity.as_item_entity().unwrap() });
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
pub fn check_player_finished(ecs: &mut World) -> bool {
    let players = ecs.read_storage::<Player>();
    let mut finished_activities = ecs.write_storage::<FinishedActivity>();
    (&players, &mut finished_activities).join().next().is_some()
}

pub fn p_input_activity(ecs: &mut World, ctx: &mut BTerm) {
    if ctx.key.is_none() {
        return;
    }

    match ctx.key.unwrap() {
        VKC::Space => {
            info!("Player pressed game action");
            let mut game_actions = ecs.write_storage::<GameAction>();
            let player_entity = ecs.read_resource::<PlayerEntity>();
            let _ = game_actions.insert(player_entity.0, GameAction {});
        }
        _ => {}
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MenuSelection {
    NewGame,
    LoadGame,
    Settings,
}

impl MenuSelection {
    /// Constructs a new string of the variant in lowercase with spaces
    pub fn to_lowercase(&self) -> String {
        match self {
            MenuSelection::NewGame => "new game",
            MenuSelection::LoadGame => "load game",
            MenuSelection::Settings => "settings",
        }
        .to_string()
    }
}

pub enum MenuAction {
    Selected(MenuSelection),
    Hovering(MenuSelection),
    Waiting,
}

pub fn p_input_main_menu(ctx: &mut BTerm, hovering: &MenuSelection) -> MenuAction {
    if let Some(key) = ctx.key {
        match key {
            VKC::Down | VKC::S => MenuAction::Hovering(match hovering {
                MenuSelection::NewGame if save_game_exists() => MenuSelection::LoadGame,
                MenuSelection::NewGame => MenuSelection::Settings,
                MenuSelection::LoadGame => MenuSelection::Settings,
                MenuSelection::Settings => MenuSelection::NewGame,
            }),
            VKC::Up | VKC::W => MenuAction::Hovering(match hovering {
                MenuSelection::NewGame => MenuSelection::Settings,
                MenuSelection::LoadGame => MenuSelection::NewGame,
                MenuSelection::Settings if save_game_exists() => MenuSelection::LoadGame,
                MenuSelection::Settings => MenuSelection::NewGame,
            }),
            VKC::Return => MenuAction::Selected(*hovering),
            _ => MenuAction::Waiting,
        }
    } else {
        MenuAction::Waiting
    }
}

pub fn p_input_save_game(ctx: &mut BTerm) -> SaveAction {
    if ctx.key.is_none() {
        return SaveAction::Waiting;
    }
    match ctx.key.unwrap() {
        VKC::Return | VKC::S => SaveAction::Save,
        VKC::Escape => SaveAction::Cancel,
        VKC::Space => SaveAction::QuickSave,
        VKC::D => SaveAction::QuitWithoutSaving,
        _ => SaveAction::Waiting,
    }
}

pub fn p_input_settings(ctx: &mut BTerm) -> SettingsAction {
    if let Some(key) = ctx.key {
        match key {
            VKC::Return => SettingsAction::Selected,
            VKC::Escape => SettingsAction::ReturnToMainMenu,
            _ => SettingsAction::Waiting,
        }
    } else {
        SettingsAction::Waiting
    }
}
