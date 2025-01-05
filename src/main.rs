use crate::being::BeingID;
use crate::colors::initialize_printer_palette;
use crate::frame_animation::AnimationRenderer;
use crate::game_init::set_level_font;
use crate::logger::create_logger;
use crate::map::MapRes;
use crate::saveload::{SerializationHelper, SerializeMe};
use crate::ui::draw_ui;
use crate::ui::message_log::MessageLog;
use std::mem::discriminant;
use std::process::exit;
use std::time::Duration;

use audio::play_sound_effect;
use being::{GoalFindEntities, GoalMoveToEntities, HandleMoveActions, RandomMonsterMovementSystem};
use bracket_lib::geometry::Point;
use bracket_lib::terminal::{main_loop, render_draw_buffer, BError, BTerm, BTermBuilder, GameState};
use combat::{AttackActionHandler, HealActionHandler};
use config::ConfigMaster;
use crafting::HandleCraftingSystem;
use debug::{debug_info, debug_input};
use draw_sprites::{draw_sprite_layers, update_fancy_positions};
use droptables::DeathLootDrop;
use equipment::EquipActionHandler;
use fov::UpdateViewsheds;
use frame_animation::{AnimationPlay, UpdateAnimationTimers};
use game_init::initialize_new_game_world;
use items::{ConsumeHandler, ItemPickupHandler, ItemSpawnerSystem, ZeroQtyItemCleanup};
use log::{debug, error, info, warn};
use mining::{DamageSystem, RemoveDeadTiles, TileDestructionSystem};
use saveload::{cleanup_game, load_game, save_game, save_game_exists, SaveAction};
use settings::{handle_setting_selected, SettingsAction, SettingsSelection};
use specs::prelude::*;

mod audio;
mod camera;
mod colors;
mod combat;
mod config;
mod data_read;
mod debug;
mod draw_sprites;
mod droptables;
mod equipment;
mod fov;
mod frame_animation;
mod game_init;
mod indexing;
mod inventory;
mod logger;
mod saveload;
mod settings;
mod storage_utils;
mod ui;
use inventory::{handle_one_item_actions, handle_two_item_actions, p_input_inventory, InventoryResponse};
mod being;
mod items;
mod map_gen;
mod mining;
mod noise;
mod player;
mod stats;
mod tile_animation;
mod z_order;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use tile_animation::{TileAnimationCleanUpSystem, TileAnimationUpdater};
mod time;
use player::{
    check_player_finished, p_input_activity, p_input_game, p_input_main_menu, p_input_save_game, p_input_settings,
    MenuAction, MenuSelection, PlayerResponse,
};
mod map;
use map::Map;
mod components;
use components::Position;
mod crafting;
mod fishing;
use fishing::{
    CatchFishSystem, CreateFishingBubbles, FishingMinigameCheck, FishingMinigameUpdate, PollFishingTiles,
    SetupFishingActions, WaitingForFishSystem,
};
use indexing::{IndexBlockedTiles, IndexBreakableTiles, IndexFishableTiles, IndexItemTiles, IndexReset};
use tile_animation::TileAnimationSpawner;
use time::delta_time_update;

use crate::components::{
    AttackBonus, Consumable, ConsumeAction, CraftAction, EntityStats, EquipAction, Equipable, EquipmentSlots, Equipped,
    FishingMinigame, GameAction, GlyphFlash, HealAction, InBag, LevelPersistent, SizeFlexor, Viewshed,
};
use crate::{
    components::{
        AttackAction, Blocking, BreakAction, Breakable, DeleteCondition, FinishedActivity, FishAction, FishOnTheLine,
        Fishable, GoalMoverAI, Grass, HealthStats, Interactor, Item, MoveAction, Name, PickupAction, RandomWalkerAI,
        Renderable, SelectedInventoryItem, SufferDamage, Transform, WaitingForFish, Water,
    },
    data_read::initialize_game_databases,
    items::ItemSpawner,
    player::Player,
    tile_animation::TileAnimationBuilder,
    time::DeltaTime,
};

// Size of the terminal window
pub const DISPLAY_WIDTH: usize = 40;
pub const DISPLAY_HEIGHT: usize = 30;
// Double for size of ui since it's scaled down

pub struct State {
    ecs: World,
    cfg: ConfigMaster,
}

impl State {
    fn run_response_systems(&mut self) {
        let mut randomwalker = RandomMonsterMovementSystem;
        randomwalker.run_now(&self.ecs);
        let mut find_goals = GoalFindEntities;
        find_goals.run_now(&self.ecs);
        let mut goalmover = GoalMoveToEntities;
        goalmover.run_now(&self.ecs);
        let mut handle_moves = HandleMoveActions;
        handle_moves.run_now(&self.ecs);
        let mut handle_attack_actions = AttackActionHandler;
        handle_attack_actions.run_now(&self.ecs);

        let mut update_fishing_tiles = CreateFishingBubbles;
        update_fishing_tiles.run_now(&self.ecs);
    }

    fn run_activity_bound_systems(&mut self) {
        // Fishing Minigame Systems ====================>
        let mut waiting_for_fish = WaitingForFishSystem;
        waiting_for_fish.run_now(&self.ecs);
        let mut fish_mini_update = FishingMinigameUpdate;
        fish_mini_update.run_now(&self.ecs);
        let mut fish_mini_check = FishingMinigameCheck;
        fish_mini_check.run_now(&self.ecs);
        let mut catch_fish = CatchFishSystem;
        catch_fish.run_now(&self.ecs);
    }

    fn run_ingame_systems(&mut self) {
        // Indexing Systems ===============================>
        let mut index_reset = IndexReset;
        index_reset.run_now(&self.ecs);
        let mut index_blocking = IndexBlockedTiles;
        index_blocking.run_now(&self.ecs);
        let mut index_breaking = IndexBreakableTiles;
        index_breaking.run_now(&self.ecs);
        let mut index_fishing = IndexFishableTiles;
        index_fishing.run_now(&self.ecs);
        let mut index_items = IndexItemTiles;
        index_items.run_now(&self.ecs);

        // Fishing Systems ==================================>
        let mut setup_fishing_actions = SetupFishingActions;
        setup_fishing_actions.run_now(&self.ecs);
        let mut waiting_for_fish = WaitingForFishSystem;
        waiting_for_fish.run_now(&self.ecs);
        let mut catch_fish = CatchFishSystem;
        catch_fish.run_now(&self.ecs);
        let mut poll_fishing_tiles = PollFishingTiles;
        poll_fishing_tiles.run_now(&self.ecs);

        // Action Systems =================================>
        let mut heal_handler = HealActionHandler;
        heal_handler.run_now(&self.ecs);
        let mut destruction_sys = TileDestructionSystem;
        destruction_sys.run_now(&self.ecs);
        let mut damage_sys = DamageSystem;
        damage_sys.run_now(&self.ecs);
        let mut item_pickup_handler = ItemPickupHandler;
        item_pickup_handler.run_now(&self.ecs);

        // Misc Systems ==================================>
        let mut death_loot_spawn = DeathLootDrop;
        death_loot_spawn.run_now(&self.ecs);
        let mut viewshed_update = UpdateViewsheds;
        viewshed_update.run_now(&self.ecs);

        // Request Based Systems ================================>
        let mut item_spawner = ItemSpawnerSystem;
        item_spawner.run_now(&self.ecs);

        // Animation Systems =========================================>
        let mut tile_anim_spawner = TileAnimationSpawner;
        tile_anim_spawner.run_now(&self.ecs);
        let mut tile_anim_updater = TileAnimationUpdater;
        tile_anim_updater.run_now(&self.ecs);
        let mut tile_anim_cleanup_system = TileAnimationCleanUpSystem;
        tile_anim_cleanup_system.run_now(&self.ecs);

        // Cleanup Systems =======================================>
        let mut zero_qty_item_cleanup = ZeroQtyItemCleanup;
        zero_qty_item_cleanup.run_now(&self.ecs);
        let mut remove_dead_tiles = RemoveDeadTiles;
        remove_dead_tiles.run_now(&self.ecs);
    }

    /// Systems that need to be ran after most other systems are finished EOF - end of frame
    fn run_eof_systems(&mut self) {
        self.ecs.write_storage::<FinishedActivity>().clear();
    }
}

/// Defines the app's state for the game
#[derive(Clone, PartialEq, Eq)]
pub enum AppState {
    MainMenu { hovering: MenuSelection },
    SettingsMenu { hovering: SettingsSelection },
    NewGameStart,
    LoadGameStart,
    MapChange { level_name: String, player_world_pos: Position },
    InGame,
    ActivityBound { response_delay: Duration },
    PlayerInInventory,
    SaveGame,
    PreRun { next_state: Box<AppState> },
}

struct FrameState {
    current: AppState,
    next: AppState,
}

impl FrameState {
    fn new(current: &AppState) -> Self {
        Self { current: current.clone(), next: current.clone() }
    }

    /// Sets the state for the next frame
    fn change_to(&mut self, new: AppState) {
        self.next = new;
    }

    /// will change if the next state is different from the current
    pub fn will_change(&self) -> bool {
        discriminant(&self.current) != discriminant(&self.next)
    }
}

impl AppState {
    /// Creates the enum variant ActivityBound with zero duration
    pub fn activity_bound() -> Self {
        Self::ActivityBound { response_delay: Duration::ZERO }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut frame_state: FrameState;
        {
            // this is in a new scope because we need to mutate self (the ecs) later in the fn
            let current_state = self.ecs.fetch_mut::<AppState>();
            frame_state = FrameState::new(&current_state);
        }

        match frame_state.current.clone() {
            AppState::NewGameStart => {
                info!("Game startup occured");
                initialize_new_game_world(&mut self.ecs);
                set_level_font(&self.ecs, ctx);

                let mut item_spawner = ItemSpawnerSystem;
                item_spawner.run_now(&self.ecs);
                frame_state.change_to(AppState::InGame);
                info!("Game startup finished. Switching to ingame state");
            }
            AppState::LoadGameStart => {
                debug!("Attempting to load save file");
                load_game(&mut self.ecs);
                set_level_font(&self.ecs, ctx);

                frame_state.change_to(AppState::InGame);
                debug!("Loaded save file");
            }
            AppState::InGame => {
                match p_input_game(&mut self.ecs, ctx) {
                    PlayerResponse::Waiting => {
                        // Player hasn't done anything yet so only run essential systems
                    }
                    PlayerResponse::TurnAdvance => {
                        turn_counter_incr(&mut self.ecs);
                        self.run_response_systems();
                    }
                    PlayerResponse::StateChange(delta_state) => {
                        frame_state.change_to(delta_state);
                    }
                }
                self.run_ingame_systems();
                self.run_eof_systems();
            }
            AppState::PlayerInInventory => {
                match p_input_inventory(&mut self.ecs, ctx, &mut self.cfg.inventory) {
                    InventoryResponse::Waiting => {
                        // Player hasn't done anything yet so only run essential systems
                    }
                    InventoryResponse::ActionReady => {
                        handle_one_item_actions(&mut self.ecs);
                        let mut equip_system = EquipActionHandler;
                        equip_system.run_now(&self.ecs);
                        let mut consume_handler = ConsumeHandler;
                        consume_handler.run_now(&self.ecs);
                        let mut heal_handler = HealActionHandler;
                        heal_handler.run_now(&self.ecs);
                        let mut damage_sys = DamageSystem;
                        damage_sys.run_now(&self.ecs);
                    }
                    InventoryResponse::SecondItemSelected { second_item } => {
                        handle_two_item_actions(&mut self.ecs, &second_item);
                        let mut craft_system = HandleCraftingSystem;
                        craft_system.run_now(&self.ecs);
                    }
                    InventoryResponse::StateChange(delta_state) => {
                        frame_state.change_to(delta_state);
                    }
                }
                let mut item_spawner = ItemSpawnerSystem;
                item_spawner.run_now(&self.ecs);
                let mut zero_qty_item_cleanup = ZeroQtyItemCleanup;
                zero_qty_item_cleanup.run_now(&self.ecs);
            }
            AppState::ActivityBound { response_delay } => {
                p_input_activity(&mut self.ecs, ctx);
                self.run_activity_bound_systems();

                frame_state.change_to(if check_player_finished(&mut self.ecs) {
                    turn_counter_incr(&mut self.ecs);
                    self.run_response_systems();
                    AppState::InGame
                } else {
                    AppState::ActivityBound { response_delay }
                });
            }
            AppState::MapChange { level_name, player_world_pos: _ } => {
                debug!("going to {}", level_name);
                unreachable!("TODO: donot switch to mapchange state");
                // cleanup_old_map(&mut self.ecs);
                // let new_level = create_map(&mut self.ecs, &level_name);
                // self.ecs.insert(MapRes(new_level));

                // set_level_font(&self.ecs, ctx);
                // move_player_to(&player_world_pos, &mut self.ecs);
                // frame_state.change_to(AppState::InGame);
            }
            AppState::MainMenu { hovering } => {
                let mut timer_update = UpdateAnimationTimers;
                timer_update.run_now(&mut self.ecs);
                match p_input_main_menu(ctx, &hovering) {
                    MenuAction::Selected(selected) => {
                        frame_state.change_to(match selected {
                            MenuSelection::NewGame => {
                                play_sound_effect("confirm");
                                AppState::NewGameStart
                            }
                            MenuSelection::LoadGame => {
                                if save_game_exists() {
                                    play_sound_effect("confirm");
                                    AppState::LoadGameStart
                                } else {
                                    AppState::MainMenu { hovering: MenuSelection::NewGame }
                                }
                            }
                            MenuSelection::Settings => {
                                play_sound_effect("confirm");
                                AppState::SettingsMenu { hovering: SettingsSelection::SpriteMode }
                            }
                            MenuSelection::QuitGame => {
                                info!("Quitting the game from the main menu.");
                                exit(1);
                            }
                        });
                    }
                    MenuAction::Hovering(new_selection) if new_selection != hovering => {
                        frame_state.change_to(AppState::MainMenu { hovering: new_selection });
                        play_sound_effect("ui_move");
                    }
                    MenuAction::Waiting | MenuAction::Hovering(..) => {
                        // do nothing..
                    }
                }
            }
            AppState::SettingsMenu { hovering } => match p_input_settings(ctx) {
                SettingsAction::Selected => {
                    handle_setting_selected(&hovering, &mut self.cfg.general, ctx);
                }
                SettingsAction::ReturnToMainMenu => {
                    self.cfg.general.save();
                    frame_state.change_to(AppState::PreRun {
                        next_state: Box::new(AppState::MainMenu { hovering: MenuSelection::Settings }),
                    });
                }
                SettingsAction::Waiting => {}
            },
            AppState::SaveGame => match p_input_save_game(ctx) {
                SaveAction::Save => {
                    save_game(&mut self.ecs);
                    cleanup_game(&mut self.ecs);
                    frame_state.change_to(AppState::PreRun {
                        next_state: Box::new(AppState::MainMenu { hovering: MenuSelection::NewGame }),
                    });
                }
                SaveAction::Cancel => {
                    frame_state.change_to(AppState::InGame);
                }
                SaveAction::QuitWithoutSaving => {
                    cleanup_game(&mut self.ecs);
                    frame_state.change_to(AppState::PreRun {
                        next_state: Box::new(AppState::MainMenu { hovering: MenuSelection::NewGame }),
                    });
                }
                SaveAction::QuickSave => {
                    save_game(&mut self.ecs);
                    frame_state.change_to(AppState::PreRun { next_state: Box::new(AppState::InGame) });
                    self.ecs.write_resource::<MessageLog>().log("Saved game.");
                }
                SaveAction::Waiting => {}
            },
            AppState::PreRun { next_state } => {
                run_pre_state_systems(next_state.as_ref(), &mut self.ecs);
                frame_state.change_to(*next_state);
            }
        }

        // Essential Systems run every frame
        update_fancy_positions(&self.ecs);
        delta_time_update(&mut self.ecs, ctx);
        self.ecs.maintain();

        draw_ui(&self.ecs, &frame_state.current, &self.cfg);
        // NOTE: for some unknown reason this must be called before the debug info so that
        // the debug info is drawn ontop of the ui
        render_draw_buffer(ctx).expect("Render error??");

        match frame_state.current {
            AppState::InGame | AppState::PlayerInInventory | AppState::NewGameStart | AppState::MapChange { .. } => {
                draw_sprite_layers(&self.ecs);
                debug_info(ctx, &self.ecs, &self.cfg.inventory);
                debug_input(ctx, &self.ecs);
            }
            AppState::MainMenu { .. }
            | AppState::SettingsMenu { .. }
            | AppState::ActivityBound { .. }
            | AppState::SaveGame
            | AppState::LoadGameStart
            | AppState::PreRun { .. } => (),
        }

        if frame_state.will_change() {
            run_exit_state_systems(&frame_state.current, &mut self.ecs);
        }

        // Insert the state resource to overwrite it's existing and update the state of the app
        let mut state_writer = self.ecs.write_resource::<AppState>();
        *state_writer = frame_state.next;
    }
}

/// These systems are ran every time upon entering a state
fn run_pre_state_systems(state: &AppState, ecs: &mut World) {
    match state {
        AppState::MainMenu { .. } => {
            ecs.write_resource::<AnimationRenderer>()
                .request("main_menu_intro", AnimationPlay::lasting(Point::new(DISPLAY_WIDTH / 4, DISPLAY_HEIGHT / 3)));
        }
        _ => {}
    }
}

/// These systems are ran every time a state is changed
fn run_exit_state_systems(state: &AppState, ecs: &mut World) {
    match state {
        AppState::MainMenu { .. } => {
            ecs.write_resource::<AnimationRenderer>().clear();
        }
        _ => {}
    }
}

struct TurnCounter(pub usize);
impl TurnCounter {
    pub fn zero() -> Self {
        Self(0)
    }
}

fn turn_counter_incr(ecs: &mut World) {
    let mut tc = ecs.fetch_mut::<TurnCounter>();
    tc.0 += 1;
}

// CL - Console layer, represents the indices for each console
pub const CL_EFFECTS2: usize = 3; // Used for special effect tiles on top of other effects
pub const CL_EFFECTS: usize = 2; // Used for special effect tiles
pub const CL_TEXT: usize = 4; // Used for UI
pub const CL_WORLD: usize = 0; // Used for terrain tiles
pub const CL_INTERACTABLES: usize = 1; // Used for the few or so moving items/entities on screen

// FONTS - the indices are based on the order the fonts are added in the context init
pub const FONT_EFFECTS: usize = 0;
pub const FONT_TEXT: usize = 1;
pub const FONT_INTERACTABLES: usize = 2;
pub const FONT_INTERACTABLES_OUTLINE: usize = 3;
pub const FONT_TERRAIN_FOREST: usize = 4;
pub const FONT_TERRAIN_TOWN_FOREST: usize = 5;

// embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
// embedded_resource!(TILE_OUTLINE_FONT, "../resources/interactable_tiles_outline.png");
// embedded_resource!(TILE_EFFECT, "../resources/effects_tiles.png");
// embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");
// embedded_resource!(TERRAIN_FOREST, "../resources/terrain_forest.png");
// embedded_resource!(TERRAIN_TOWN_FOREST, "../resources/terrain_town_forest.png");
// embedded_resource!(LDTK_WORLD, "../resources/ldtk/rpg_world_v2.ldtk");

fn main() -> BError {
    // TODO: setup for release builds
    // link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    // link_resource!(TILE_OUTLINE_FONT, "resources/interactable_tiles_outline.png");
    // link_resource!(TILE_EFFECT, "resources/effects_tiles.png");
    // link_resource!(CHAR_FONT, "resources/terminal8x8.png");
    // link_resource!(TERRAIN_FOREST, "resources/terrain_forest.png");
    // link_resource!(TERRAIN_TOWN_FOREST, "resources/terrain_town_forest.png");
    // link_resource!(LDTK_WORLD, "../resources/ldtk/rpg_world_v2.ldtk");

    create_logger();
    info!("Info will be tracked in this file.");
    error!("Errors will be tracked in this file.");
    warn!("Warnings will be tracked in this file.");

    initialize_game_databases();

    let cfg = ConfigMaster::load();
    let interactable_font = match cfg.general.sprite_mode {
        settings::SpriteMode::Outline => "interactable_tiles_outline.png",
        settings::SpriteMode::Blocked => "interactable_tiles.png",
    };
    let text_font = "zaratustra.png";

    // Setup Terminal (incl Window, Input, Font Loading)
    let mut context = BTermBuilder::new()
        .with_title("RPG")
        .with_fps_cap(60.0)
        .with_font("effects_tiles.png", 8u32, 8u32)
        .with_font("zaratustra.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_font("interactable_tiles_outline.png", 8u32, 8u32)
        .with_font("terrain_forest.png", 8u32, 8u32)
        .with_font("terrain_town_forest.png", 8u32, 8u32)
        .with_dimensions(160, 120)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terrain_forest.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, interactable_font)
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "effects_tiles.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "effects_tiles.png")
        .with_fancy_console(DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2, text_font)
        .build()?;
    context.cls();

    initialize_printer_palette();

    // Setup ECS
    let mut world = World::new();

    // Component Registration, the ECS needs to have every type of component registered
    world.register::<Position>();
    world.register::<Player>();
    world.register::<BeingID>();
    world.register::<Renderable>();
    world.register::<Blocking>();
    world.register::<HealthStats>();
    world.register::<BreakAction>();
    world.register::<AttackAction>();
    world.register::<PickupAction>();
    world.register::<FishAction>();
    world.register::<Breakable>();
    world.register::<SufferDamage>();
    world.register::<Fishable>();
    world.register::<WaitingForFish>();
    world.register::<FishOnTheLine>();
    world.register::<DeleteCondition>();
    world.register::<FinishedActivity>();
    world.register::<Name>();
    world.register::<RandomWalkerAI>();
    world.register::<GoalMoverAI>();
    world.register::<Item>();
    world.register::<Water>();
    world.register::<Grass>();
    world.register::<InBag>();
    world.register::<MoveAction>();
    world.register::<CraftAction>();
    world.register::<EquipAction>();
    world.register::<Transform>();
    world.register::<Interactor>();
    world.register::<EntityStats>();
    world.register::<SelectedInventoryItem>();
    world.register::<EquipmentSlots>();
    world.register::<Equipable>();
    world.register::<Equipped>();
    world.register::<AttackBonus>();
    world.register::<Consumable>();
    world.register::<ConsumeAction>();
    world.register::<HealAction>();
    world.register::<GameAction>();
    world.register::<FishingMinigame>();
    world.register::<LevelPersistent>();
    world.register::<SizeFlexor>();
    world.register::<GlyphFlash>();
    world.register::<Viewshed>();

    // Still components but used for saving the data in the ecs
    world.register::<SimpleMarker<SerializeMe>>();
    world.register::<SerializationHelper>();
    world.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    // Resource Initialization, the ECS needs a basic definition of every resource that will be in the game
    world.insert(AppState::PreRun { next_state: Box::new(AppState::MainMenu { hovering: MenuSelection::NewGame }) });
    world.insert(DeltaTime(Duration::ZERO));
    world.insert(TileAnimationBuilder::new());
    world.insert(AnimationRenderer::new());
    world.insert(ItemSpawner::new());
    world.insert(MessageLog::new());
    world.insert(MapRes(Map::empty(0, 0)));
    world.insert(TurnCounter::zero());

    let game_state = State { ecs: world, cfg };
    main_loop(context, game_state)
}
