use crate::game_init::{load_map, cleanup_old_map, move_player_to};
use crate::logger::create_logger;
use crate::ui::draw_ui;
use crate::ui::message_log::MessageLog;
use std::time::Duration;

use being::{GoalFindEntities, GoalMoveToEntities, HandleMoveActions, RandomMonsterMovementSystem};
use bracket_terminal::prelude::*;
use combat::{AttackActionHandler, HealActionHandler};
use config::ConfigMaster;
use crafting::HandleCraftingSystem;
use debug::{debug_info, debug_input};
use draw_sprites::{draw_sprite_layers, update_fancy_positions};
use equipment::EquipActionHandler;
use game_init::initialize_new_game_world;
use items::{ConsumeHandler, ItemPickupHandler, ItemSpawnerSystem, ZeroQtyItemCleanup};
use log::{debug, error, info, warn};
use mining::{DamageSystem, RemoveDeadTiles, TileDestructionSystem};
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
mod logger;
mod storage_utils;
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
use player::{
    check_player_activity_input, check_player_finished, manage_player_input, PlayerResponse,
};
mod map;
use map::Map;
mod components;
use components::Position;
mod crafting;
mod fishing;

use fishing::{
    CatchFishSystem, FishingMinigameCheck, FishingMinigameUpdate, PollFishingTiles,
    SetupFishingActions, UpdateFishingTiles, WaitingForFishSystem,
};
use indexing::{
    IndexBlockedTiles, IndexBreakableTiles, IndexFishableTiles, IndexItemTiles, IndexReset,
};
use tile_animation::TileAnimationSpawner;
use time::delta_time_update;

use crate::components::{
    AttackBonus, Consumable, ConsumeAction, CraftAction, DeathDrop, EntityStats, EquipAction,
    Equipable, EquipmentSlots, Equipped, FishingMinigame, GameAction, HealAction, InBag,
    ItemContainer, Persistent,
};
use crate::{
    components::{
        AttackAction, Blocking, BreakAction, Breakable, DeleteCondition, FinishedActivity,
        FishAction, FishOnTheLine, Fishable, GoalMoverAI, Grass, HealthStats, Interactor, Item,
        MoveAction, Name, PickupAction, RandomWalkerAI, Renderable, SelectedInventoryItem,
        SufferDamage, Transform, WaitingForFish, Water,
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

        let mut update_fishing_tiles = UpdateFishingTiles;
        update_fishing_tiles.run_now(&self.ecs);
    }

    fn run_continuous_systems(&mut self, _ctx: &mut BTerm) {
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
        let mut fish_mini_update = FishingMinigameUpdate;
        fish_mini_update.run_now(&self.ecs);
        let mut fish_mini_check = FishingMinigameCheck;
        fish_mini_check.run_now(&self.ecs);
        let mut catch_fish = CatchFishSystem;
        catch_fish.run_now(&self.ecs);
        let mut poll_fishing_tiles = PollFishingTiles;
        poll_fishing_tiles.run_now(&self.ecs);

        // Misc Systems ==================================>
        let mut heal_handler = HealActionHandler;
        heal_handler.run_now(&self.ecs);
        let mut destruction_sys = TileDestructionSystem;
        destruction_sys.run_now(&self.ecs);
        let mut damage_sys = DamageSystem;
        damage_sys.run_now(&self.ecs);
        let mut item_pickup_handler = ItemPickupHandler;
        item_pickup_handler.run_now(&self.ecs);
        let mut item_spawner = ItemSpawnerSystem;
        item_spawner.run_now(&self.ecs);

        // Animation Systems =========================================>
        let mut tile_anim_spawner = TileAnimationSpawner;
        tile_anim_spawner.run_now(&self.ecs);
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
#[derive(Clone)]
pub enum AppState {
    GameStartup,
    MapChange { level_name: String, player_world_pos: Position},
    InGame,
    ActivityBound { response_delay: Duration }, // can only perform a specific acitivity that is currently happening
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
            let current_state = self.ecs.fetch_mut::<AppState>();
            new_state = current_state.clone();
        }

        match new_state {
            AppState::GameStartup => {
                info!("Game startup occured");
                initialize_new_game_world(&mut self.ecs, ctx);
                let mut item_spawner = ItemSpawnerSystem;
                item_spawner.run_now(&self.ecs);
                new_state = AppState::InGame;
                info!("Game startup finished. Switching to ingame state");
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
                        new_state = delta_state;
                    }
                }
                let mut item_spawner = ItemSpawnerSystem;
                item_spawner.run_now(&self.ecs);
                let mut zero_qty_item_cleanup = ZeroQtyItemCleanup;
                zero_qty_item_cleanup.run_now(&self.ecs);
            }
            AppState::ActivityBound { response_delay } => {
                check_player_activity_input(&mut self.ecs, ctx);
                self.run_continuous_systems(ctx);

                new_state = if check_player_finished(&mut self.ecs) {
                    turn_counter_incr(&mut self.ecs);
                    self.run_response_systems();
                    AppState::InGame
                } else {
                    AppState::ActivityBound { response_delay }
                };

                self.run_eof_systems();
            }
            AppState::MapChange { level_name, player_world_pos } => {
                debug!("going to {}", level_name);
                cleanup_old_map(&mut self.ecs);
                load_map(&level_name, &mut self.ecs, ctx);
                move_player_to(&player_world_pos, &mut self.ecs);
                new_state = AppState::InGame;
            }
        }

        // Essential Systems ran every frame
        update_fancy_positions(&self.ecs);
        delta_time_update(&mut self.ecs, ctx);
        self.ecs.maintain();

        draw_ui(&self.ecs, &new_state, &self.cfg.inventory);
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
pub const FONT_TERRAIN_FOREST: usize = 3;
pub const FONT_TERRAIN_TOWN_FOREST: usize = 4;

embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
embedded_resource!(TILE_EFFECT, "../resources/effects_tiles.png");
embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");
embedded_resource!(TERRAIN_FOREST, "../resources/terrain_forest.png");
embedded_resource!(TERRAIN_TOWN_FOREST, "../resources/terrain_town_forest.png");
embedded_resource!(LDTK_WORLD, "../resources/ldtk/rpg_world_v2.ldtk");

fn main() -> BError {
    link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    link_resource!(TILE_EFFECT, "resources/effects_tiles.png");
    link_resource!(CHAR_FONT, "resources/terminal8x8.png");
    link_resource!(TERRAIN_FOREST, "resources/terrain_forest.png");
    link_resource!(TERRAIN_TOWN_FOREST, "resources/terrain_town_forest.png");
    link_resource!(LDTK_WORLD, "../resources/ldtk/rpg_world_v2.ldtk");

    create_logger();
    info!("Info will be tracked in this file.");
    error!("Errors will be tracked in this file.");
    warn!("Warnings will be tracked in this file.");

    initialize_game_databases();

    // Setup Terminal (incl Window, Input, Font Loading)
    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_fps_cap(60.0)
        .with_font("effects_tiles.png", 8u32, 8u32)
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_font("terrain_forest.png", 8u32, 8u32)
        .with_font("terrain_town_forest.png", 8u32, 8u32)
        .with_dimensions(160, 120)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terrain_forest.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "interactable_tiles.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "effects_tiles.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "effects_tiles.png")
        .with_fancy_console(DISPLAY_WIDTH * 2, DISPLAY_HEIGHT * 2, "terminal8x8.png")
        .build()?;

    register_palette_color("orange", RGB::from_u8(230, 113, 70));
    register_palette_color("red", RGB::from_u8(183, 65, 50));
    register_palette_color("bright_green", RGB::from_u8(52, 156, 88));
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
    world.register::<RandomWalkerAI>();
    world.register::<GoalMoverAI>();
    world.register::<DeathDrop>();
    world.register::<Item>();
    world.register::<Water>();
    world.register::<Grass>();
    world.register::<InBag>();
    world.register::<ItemContainer>();
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
    world.register::<Persistent>();

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
