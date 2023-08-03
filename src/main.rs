use std::time::Duration;

use activity_finishing::ActivityFinishSystem;
use bracket_terminal::prelude::*;
use draw_sprites::draw_all_layers;
use mining::{DamageSystem, RemoveDeadTiles, TileDestructionSystem};
use specs::prelude::*;

mod draw_sprites;
mod mining;
mod player;
mod indexing;
mod tile_animation;
mod time;
mod activity_finishing;
use player::manage_player_input;
mod map;
use map::Map;
mod components;
use components::Position;
mod fishing;
use fishing::{SetupFishingActions, WaitingForFishSystem, CatchFishSystem};
use indexing::{IndexBlockedTiles, IndexBreakableTiles, IndexReset, IndexFishableTiles};
use tile_animation::TileAnimationSpawner;
use time::delta_time_update;

use crate::{
    components::{
        Blocking, BreakAction, Breakable, HealthStats, Renderable, Strength, SufferDamage, Fishable, FishAction, WaitingForFish, FishOnTheLine, DeleteCondition, FinishedActivity,
    },
    draw_sprites::debug_rocks,
    player::Player, map::WorldTile, time::DeltaTime, tile_animation::TileAnimationBuilder,
};

// Size of the terminal window
pub const DISPLAY_WIDTH: u32 = 40;
pub const DISPLAY_HEIGHT: u32 = 30;

// CL - Console layer, represents the indices for each console
pub const CL_TEXT: usize = 2; // Used for UI
pub const CL_WORLD: usize = 0; // Used for terrain tiles
pub const CL_INTERACTABLES: usize = 1; // Used for the few or so moving items/entities on screen

pub struct State {
    ecs: World,
    _player_state: PlayerState,
}

/// Defines the player's state for the game
pub enum PlayerState {
    InMenu,
    WaitingForInput,
    RespondingToInput,
    ActivityBound, // can only perform a specific acitivity that is currently happening
}

impl State {
    fn run_systems(&mut self, _ctx: &mut BTerm) {
        // Indexing systems
        let mut indexreset = IndexReset;
        indexreset.run_now(&self.ecs);
        let mut indexblocking = IndexBlockedTiles;
        indexblocking.run_now(&self.ecs);
        let mut indexbreaking = IndexBreakableTiles;
        indexbreaking.run_now(&self.ecs);
        let mut indexfishing = IndexFishableTiles;
        indexfishing.run_now(&self.ecs);

        let mut setupfishingactions = SetupFishingActions;
        setupfishingactions.run_now(&self.ecs);
        let mut waitingforfishsystem = WaitingForFishSystem;
        waitingforfishsystem.run_now(&self.ecs);
        let mut catchfishsystem = CatchFishSystem;
        catchfishsystem.run_now(&self.ecs);

        let mut mining_sys = TileDestructionSystem;
        mining_sys.run_now(&self.ecs);
        let mut damage_sys = DamageSystem;
        damage_sys.run_now(&self.ecs);

        // Request based system run as late as possible in the loop
        let mut tile_anim_spawner = TileAnimationSpawner {world: &self.ecs};
        tile_anim_spawner.run_now(&self.ecs);

        let mut activity_finish_system = ActivityFinishSystem;
        activity_finish_system.run_now(&self.ecs);

        let mut remove_dead_tiles = RemoveDeadTiles;
        remove_dead_tiles.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        manage_player_input(self, ctx);
        self.run_systems(ctx);
        delta_time_update(&mut self.ecs, ctx);
        draw_all_layers(&self.ecs, ctx);
    }
}

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
bracket_terminal::embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");
bracket_terminal::embedded_resource!(TERRAIN_FOREST, "../resources/terrain_forest.png");

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    bracket_terminal::link_resource!(CHAR_FONT, "resources/terminal8x8.png");
    bracket_terminal::link_resource!(TERRAIN_FOREST, "resources/terrain_forest.png");

    // Setup Terminal (incl Window, Input, Font Loading)
    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_fps_cap(60.0)
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_font("terrain_forest.png", 8u32, 8u32)
        .with_dimensions(DISPLAY_WIDTH * 3, DISPLAY_HEIGHT * 3)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terrain_forest.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "interactable_tiles.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        .build()?;

    // Setup ECS
    let mut world = World::new();
    // Component Registration, the ECS needs to have every type of component registered
    world.register::<Position>();
    world.register::<Player>();
    world.register::<Renderable>();
    world.register::<Blocking>();
    world.register::<HealthStats>();
    world.register::<BreakAction>();
    world.register::<Breakable>();
    world.register::<SufferDamage>();
    world.register::<Strength>();
    world.register::<Fishable>();
    world.register::<FishAction>();
    world.register::<WaitingForFish>();
    world.register::<FishOnTheLine>();
    world.register::<DeleteCondition>();
    world.register::<FinishedActivity>();

    // Resource Initialization, the ECS needs a basic definition of every resource
    world.insert(DeltaTime(Duration::ZERO));
    world.insert(TileAnimationBuilder::new());

    // A very plain map
    let mut map = Map::new(DISPLAY_WIDTH as usize, DISPLAY_HEIGHT as usize);
    let water_idx = map.xy_to_idx(10, 15);
    map.tiles[water_idx] = WorldTile { atlas_index: 80 };
    world.create_entity()
        .with(Position::new(10, 15))
        .with(Fishable)
        .with(Blocking)
        .build();

    world.insert(map);

    world
        .create_entity()
        .with(Position::new(17, 20))
        .with(Player {})
        .with(Strength { amt: 1 })
        .with(Renderable::new(ColorPair::new(WHITE, BLACK), 2))
        .with(Blocking)
        .build();

    debug_rocks(&mut world);

    let game_state: State = State { ecs: world, _player_state: PlayerState::WaitingForInput };
    main_loop(context, game_state)
}
