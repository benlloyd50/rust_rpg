use crate::ui::draw_ui;
use crate::ui::message_log::MessageLog;
use std::fs::{self, OpenOptions};
use std::time::Duration;

use being::{
    check_monster_delay, GoalFindEntities, GoalMoveToEntities, HandleMoveActions,
    RandomMonsterMovementSystem,
};
use bracket_terminal::prelude::*;
use combat::AttackActionHandler;
use config::ConfigMaster;
use crafting::HandleCraftingSystem;
use debug::{debug_info, debug_input};
use draw_sprites::{draw_sprite_layers, update_fancy_positions};
use equipment::EquipActionHandler;
use game_init::initialize_game_world;
use items::{ItemPickupHandler, ItemSpawnerSystem, ZeroQtyItemCleanup};
use log::{error, info, warn, LevelFilter};
use mining::{DamageSystem, RemoveDeadTiles, TileDestructionSystem};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use specs::prelude::*;

mod camera;
mod colors;
mod combat;
mod config;
mod data_read;
mod debug;
mod draw_sprites;
mod droptables;
mod equipment;
mod game_init;
mod indexing;
mod inventory;
mod ui;
use inventory::{
    handle_one_item_actions, handle_player_input, handle_two_item_actions, InventoryResponse,
};
mod being;
mod items;
mod mining;
mod player;
mod stats;
mod tile_animation;
mod z_order;
use tile_animation::TileAnimationCleanUpSystem;
mod time;
use player::{check_player_activity, manage_player_input, PlayerResponse};
mod map;
use map::Map;
mod components;
use components::Position;
mod crafting;
mod fishing;

use fishing::{
    CatchFishSystem, PollFishingTiles, SetupFishingActions, UpdateFishingTiles,
    WaitingForFishSystem,
};
use indexing::{
    IndexBlockedTiles, IndexBreakableTiles, IndexFishableTiles, IndexItemTiles, IndexReset,
};
use tile_animation::TileAnimationSpawner;
use time::delta_time_update;

use crate::components::{
    AttackBonus, DeathDrop, EntityStats, Equipable, EquipmentSlots, Equipped, InBag, ItemContainer,
    WantsToCraft, WantsToEquip,
};
use crate::{
    components::{
        AttackAction, Blocking, BreakAction, Breakable, DeleteCondition, FinishedActivity,
        FishAction, FishOnTheLine, Fishable, GoalMoverAI, Grass, HealthStats, Interactor, Item,
        Monster, Name, PickupAction, RandomWalkerAI, Renderable, SelectedInventoryItem,
        SufferDamage, Transform, WaitingForFish, WantsToMove, Water,
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

// CL - Console layer, represents the indices for each console
pub const CL_TEXT: usize = 2; // Used for UI
pub const CL_WORLD: usize = 0; // Used for terrain tiles
pub const CL_INTERACTABLES: usize = 1; // Used for the few or so moving items/entities on screen

pub struct State {
    ecs: World,
    cfg: ConfigMaster,
}

impl State {
    fn run_response_systems(&mut self) {
        // println!("Response Systems are now running.");
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

        let mut update_fishing_tiles = UpdateFishingTiles;
        update_fishing_tiles.run_now(&self.ecs);
        // println!("Response Systems are now finished.");
    }

    fn run_continuous_systems(&mut self, _ctx: &mut BTerm) {
        // println!("Continuous Systems are now running.");
        // Indexing systems
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

        let mut setup_fishing_actions = SetupFishingActions;
        setup_fishing_actions.run_now(&self.ecs);
        let mut waiting_for_fish = WaitingForFishSystem;
        waiting_for_fish.run_now(&self.ecs);
        let mut catch_fish = CatchFishSystem;
        catch_fish.run_now(&self.ecs);
        let mut poll_fishing_tiles = PollFishingTiles;
        poll_fishing_tiles.run_now(&self.ecs);

        let mut mining_sys = TileDestructionSystem;
        mining_sys.run_now(&self.ecs);
        let mut damage_sys = DamageSystem;
        damage_sys.run_now(&self.ecs);
        let mut item_pickup_handler = ItemPickupHandler;
        item_pickup_handler.run_now(&self.ecs);

        // Request based system run as late as possible in the loop
        let mut tile_anim_spawner = TileAnimationSpawner;
        tile_anim_spawner.run_now(&self.ecs);
        let mut tile_anim_cleanup_system = TileAnimationCleanUpSystem;
        tile_anim_cleanup_system.run_now(&self.ecs);

        let mut remove_dead_tiles = RemoveDeadTiles;
        remove_dead_tiles.run_now(&self.ecs);

        let mut item_spawner = ItemSpawnerSystem;
        item_spawner.run_now(&self.ecs);
        let mut zero_qty_item_cleanup = ZeroQtyItemCleanup;
        zero_qty_item_cleanup.run_now(&self.ecs);
        // println!("Continuous Systems are now finished.");
    }

    /// Systems that need to be ran after most other systems are finished EOF - end of frame
    fn run_eof_systems(&mut self) {
        self.ecs.write_storage::<FinishedActivity>().clear();
    }
}

/// Defines the app's state for the game
#[derive(Clone, Copy)]
pub enum AppState {
    InMenu,
    InGame,
    ActivityBound { response_delay: Duration }, // can only perform a specific acitivity that is currently happening
    GameStartup,
    PlayerInInventory,
}

impl AppState {
    /// Creates the enum variant ActivityBound with zero duration
    pub fn activity_bound() -> Self {
        Self::ActivityBound {
            response_delay: Duration::ZERO,
        }
    }
}

fn turn_counter_incr(ecs: &mut World) {
    let mut tc = ecs.fetch_mut::<TurnCounter>();
    tc.0 += 1;
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut new_state: AppState;
        {
            // this is in a new scope because we need to mutate self (the ecs) later in the fn
            let current_state = self.ecs.fetch::<AppState>();
            new_state = *current_state;
        }

        match new_state {
            AppState::GameStartup => {
                initialize_game_world(&mut self.ecs);
                new_state = AppState::InGame;
            }
            AppState::InMenu => {
                todo!("player input will control the menu, when menus are implemented")
            }
            AppState::InGame => {
                match manage_player_input(&mut self.ecs, ctx) {
                    PlayerResponse::Waiting => {
                        // Player hasn't done anything yet so only run essential systems
                    }
                    PlayerResponse::TurnAdvance => {
                        turn_counter_incr(&mut self.ecs);
                        self.run_response_systems();
                    }
                    PlayerResponse::StateChange(delta_state) => {
                        new_state = delta_state;
                    }
                }
                self.run_continuous_systems(ctx);
                self.run_eof_systems();
            }
            AppState::PlayerInInventory => {
                match handle_player_input(&mut self.ecs, ctx, &mut self.cfg.inventory) {
                    InventoryResponse::Waiting => {
                        // Player hasn't done anything yet so only run essential systems
                    }
                    InventoryResponse::ActionReady => {
                        handle_one_item_actions(&mut self.ecs);
                        let mut equip_system = EquipActionHandler;
                        equip_system.run_now(&self.ecs);
                    }
                    InventoryResponse::SecondItemSelected { second_item } => {
                        handle_two_item_actions(&mut self.ecs, &second_item);
                        let mut craft_system = HandleCraftingSystem;
                        craft_system.run_now(&self.ecs);
                        let mut item_spawner = ItemSpawnerSystem;
                        item_spawner.run_now(&self.ecs);
                        let mut zero_qty_item_cleanup = ZeroQtyItemCleanup;
                        zero_qty_item_cleanup.run_now(&self.ecs);
                    }
                    InventoryResponse::StateChange(delta_state) => {
                        new_state = delta_state;
                    }
                }
            }
            AppState::ActivityBound { mut response_delay } => {
                // if the player finishes we run final systems and change state
                self.run_continuous_systems(ctx);
                new_state = if check_player_activity(&mut self.ecs) {
                    AppState::InGame
                } else if check_monster_delay(&self.ecs, &mut response_delay) {
                    // if the monster delay timer is past its due then monsters do their thing
                    self.run_response_systems();
                    AppState::activity_bound()
                } else {
                    AppState::ActivityBound { response_delay }
                };

                self.run_eof_systems();
            }
        }

        // Essential Systems ran every frame
        delta_time_update(&mut self.ecs, ctx);
        self.ecs.maintain();

        draw_ui(&self.ecs, &new_state, &self.cfg.inventory);
        update_fancy_positions(&self.ecs);
        draw_sprite_layers(&self.ecs);
        render_draw_buffer(ctx).expect("Render error??");
        debug_info(ctx, &self.ecs, &self.cfg.inventory);
        debug_input(ctx, &self.ecs);

        // Insert the state resource to overwrite it's existing and update the state of the app
        let mut state_writer = self.ecs.write_resource::<AppState>();
        *state_writer = new_state;
    }
}

struct TurnCounter(pub usize);
impl TurnCounter {
    pub fn zero() -> Self {
        Self(0)
    }
}

const LOG_FOLDER: &str = "logs/";
fn create_logger() {
    let _ = fs::create_dir(LOG_FOLDER);
    let path = format!("{}/last_run.rpglog", LOG_FOLDER);
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
        let file_logger = WriteLogger::new(log::LevelFilter::Info, Config::default(), file);
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            file_logger,
        ])
        .unwrap();
        info!("Logger initialized");
    } else {
        println!("Logging initialization failure");
    }
}

embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");
embedded_resource!(TERRAIN_FOREST, "../resources/terrain_forest.png");
embedded_resource!(LEVEL_0, "../resources/ldtk/rpg_world_v1.ldtk");

fn main() -> BError {
    link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    link_resource!(CHAR_FONT, "resources/terminal8x8.png");
    link_resource!(TERRAIN_FOREST, "resources/terrain_forest.png");
    link_resource!(LEVEL_0, "../resources/ldtk/rpg_world_v1.ldtk");

    create_logger();
    info!("Info will be tracked in this file.");
    error!("Errors will be tracked in this file.");
    warn!("Warnings will be tracked in this file.");

    initialize_game_databases();

    // Setup Terminal (incl Window, Input, Font Loading)
    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_fps_cap(60.0)
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_font("terrain_forest.png", 8u32, 8u32)
        .with_dimensions(160, 120)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terrain_forest.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "interactable_tiles.png")
        .with_fancy_console(DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2, "terminal8x8.png")
        .build()?;

    register_palette_color("orange", RGB::from_u8(209, 95, 38));
    register_palette_color("white", RGB::from_u8(222, 222, 222));
    register_palette_color("lightgray", RGB::from_u8(161, 161, 161));

    // Setup ECS
    let mut world = World::new();
    // Component Registration, the ECS needs to have every type of component registered
    world.register::<Position>();
    world.register::<Player>();
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
    world.register::<Monster>();
    world.register::<RandomWalkerAI>();
    world.register::<GoalMoverAI>();
    world.register::<DeathDrop>();
    world.register::<Item>();
    world.register::<Water>();
    world.register::<Grass>();
    world.register::<InBag>();
    world.register::<ItemContainer>();
    world.register::<WantsToMove>();
    world.register::<WantsToCraft>();
    world.register::<WantsToEquip>();
    world.register::<Transform>();
    world.register::<Interactor>();
    world.register::<EntityStats>();
    world.register::<SelectedInventoryItem>();
    world.register::<EquipmentSlots>();
    world.register::<Equipable>();
    world.register::<Equipped>();
    world.register::<AttackBonus>();

    // Resource Initialization, the ECS needs a basic definition of every resource that will be in the game
    world.insert(AppState::GameStartup);
    world.insert(DeltaTime(Duration::ZERO));
    world.insert(TileAnimationBuilder::new());
    world.insert(ItemSpawner::new());
    world.insert(MessageLog::new());
    world.insert(Map::empty());
    world.insert(TurnCounter::zero());

    let game_state: State = State {
        ecs: world,
        cfg: ConfigMaster::default(),
    };
    main_loop(context, game_state)
}
