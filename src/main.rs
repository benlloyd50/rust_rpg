use std::time::Duration;

use bracket_terminal::prelude::*;
use draw_sprites::draw_sprite_layers;
use game_init::initialize_game_world;
use items::{ItemPickupHandler, ItemSpawnerSystem};
use mining::{DamageSystem, RemoveDeadTiles, TileDestructionSystem};
use monster::{check_monster_delay, RandomMonsterMovementSystem};
use specs::prelude::*;

mod camera;
mod data_read;
mod draw_sprites;
mod game_init;
mod indexing;
mod items;
mod message_log;
mod mining;
mod monster;
mod player;
mod tile_animation;
mod user_interface;
use tile_animation::TileAnimationCleanUpSystem;
mod time;
use player::{check_player_activity, manage_player_input, PlayerResponse};
mod map;
use map::Map;
mod components;
use components::Position;
mod fishing;
use fishing::{CatchFishSystem, SetupFishingActions, UpdateFishingTiles, WaitingForFishSystem};
use indexing::{
    IndexBlockedTiles, IndexBreakableTiles, IndexFishableTiles, IndexItemTiles, IndexReset,
};
use tile_animation::TileAnimationSpawner;
use time::delta_time_update;
use user_interface::draw_ui;

use crate::{
    components::{
        Blocking, BreakAction, Breakable, DeathDrop, DeleteCondition, FinishedActivity, FishAction,
        FishOnTheLine, Fishable, HealthStats, InBackpack, Item, Monster, Name, PickupAction,
        RandomWalkerAI, Renderable, Strength, SufferDamage, WaitingForFish, Water,
    },
    data_read::initialize_game_databases,
    items::ItemSpawner,
    message_log::MessageLog,
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
}

impl State {
    fn run_response_systems(&mut self) {
        // println!("Response Systems are now running.");
        let mut randomwalker = RandomMonsterMovementSystem;
        randomwalker.run_now(&self.ecs);
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

        let mut update_fishing_tiles = UpdateFishingTiles;
        update_fishing_tiles.run_now(&self.ecs);
        let mut setup_fishing_actions = SetupFishingActions;
        setup_fishing_actions.run_now(&self.ecs);
        let mut waiting_for_fish = WaitingForFishSystem;
        waiting_for_fish.run_now(&self.ecs);
        let mut catch_fish = CatchFishSystem;
        catch_fish.run_now(&self.ecs);

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
}

impl AppState {
    /// Creates the enum variant ActivityBound with zero duration
    pub fn activity_bound() -> Self {
        Self::ActivityBound {
            response_delay: Duration::ZERO,
        }
    }
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
                // if we have to run something before player put it here >>>
                match manage_player_input(self, ctx) {
                    PlayerResponse::Waiting => {
                        // Player hasn't done anything yet so only run essential systems
                    }
                    PlayerResponse::TurnAdvance => {
                        self.run_response_systems();
                    }
                    PlayerResponse::StateChange(delta_state) => {
                        new_state = delta_state;
                    }
                }
                self.run_continuous_systems(ctx);
                self.run_eof_systems();
                delta_time_update(&mut self.ecs, ctx);
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
                delta_time_update(&mut self.ecs, ctx);
            }
        }

        self.ecs.maintain();

        draw_ui(&self.ecs);
        draw_sprite_layers(&self.ecs);
        render_draw_buffer(ctx).expect("Render error??");

        // Insert the state resource to overwrite it's existing and update the state of the app
        let mut state_writer = self.ecs.write_resource::<AppState>();
        *state_writer = new_state;
    }
}

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
bracket_terminal::embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");
bracket_terminal::embedded_resource!(TERRAIN_FOREST, "../resources/terrain_forest.png");
bracket_terminal::embedded_resource!(LEVEL_0, "../resources/ldtk/rpg_world_v1.ldtk");

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    bracket_terminal::link_resource!(CHAR_FONT, "resources/terminal8x8.png");
    bracket_terminal::link_resource!(TERRAIN_FOREST, "resources/terrain_forest.png");
    bracket_terminal::link_resource!(LEVEL_0, "../resources/ldtk/rpg_world_v1.ldtk");

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
    world.register::<PickupAction>();
    world.register::<Breakable>();
    world.register::<SufferDamage>();
    world.register::<Strength>();
    world.register::<Fishable>();
    world.register::<FishAction>();
    world.register::<WaitingForFish>();
    world.register::<FishOnTheLine>();
    world.register::<DeleteCondition>();
    world.register::<FinishedActivity>();
    world.register::<Name>();
    world.register::<Monster>();
    world.register::<RandomWalkerAI>();
    world.register::<DeathDrop>();
    world.register::<Item>();
    world.register::<InBackpack>();
    world.register::<Water>();

    // Resource Initialization, the ECS needs a basic definition of every resource that will be in the game
    world.insert(AppState::GameStartup);
    world.insert(DeltaTime(Duration::ZERO));
    world.insert(TileAnimationBuilder::new());
    world.insert(ItemSpawner::new());
    world.insert(MessageLog::new());
    world.insert(Map::empty());

    let game_state: State = State { ecs: world };
    main_loop(context, game_state)
}
